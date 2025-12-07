use crate::{jwt_auth::JwtMiddleware, models::ApiError, AppState};
use actix_web::{
	post,
	web::{self},
	HttpResponse,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct UpdateAdRequest {
	pub ad_id: Uuid,
	pub fields: HashMap<String, serde_json::Value>,
	pub account_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct UpdateAdResponse {
	pub ad_id: Uuid,
	pub message: String,
}

#[post("/avito/update-ad")]
pub async fn avito_update_ad(
	data: web::Data<AppState>,
	request: web::Json<UpdateAdRequest>,
	_: JwtMiddleware,
) -> Result<HttpResponse, ApiError> {
	let ad_id = request.ad_id;
	let account_id = request.account_id.unwrap_or_else(|| {
		// Default account_id if not provided - you might want to get this from JWT claims instead
		Uuid::parse_str("2acc3808-15f1-4abb-b15e-c7f4780a87da").unwrap()
	});

	dbg!(&account_id);

	// Start transaction
	let mut tx = data.db.begin().await.map_err(|e| {
		ApiError::InternalServerError(format!("Failed to start transaction: {}", e))
	})?;

	// Verify that the ad exists and belongs to the correct account
	let ad_exists = sqlx::query!(
		r#"
        SELECT a.ad_id
        FROM avito_ads a
        JOIN avito_feeds f ON a.feed_id = f.feed_id
        WHERE a.ad_id = $1 AND f.account_id = $2
        "#,
		ad_id,
		account_id
	)
	.fetch_optional(&mut *tx)
	.await
	.map_err(|e| ApiError::InternalServerError(format!("Failed to check if ad exists: {}", e)))?;

	if ad_exists.is_none() {
		return Err(ApiError::Other(
			"Ad not found or does not belong to the specified account".to_string(),
		));
	}

	// Delete existing field values and fields for this ad
	sqlx::query!(
		r#"
        DELETE FROM avito_ad_field_values
        WHERE field_id IN (
            SELECT field_id
            FROM avito_ad_fields
            WHERE ad_id = $1
        )
        "#,
		ad_id
	)
	.execute(&mut *tx)
	.await
	.map_err(|e| ApiError::InternalServerError(format!("Failed to delete field values: {}", e)))?;

	sqlx::query!(
		r#"
        DELETE FROM avito_ad_fields
        WHERE ad_id = $1
        "#,
		ad_id
	)
	.execute(&mut *tx)
	.await
	.map_err(|e| ApiError::InternalServerError(format!("Failed to delete fields: {}", e)))?;

	// Process each field in the request for batch insert
	let mut field_ids = Vec::new();
	let mut field_ad_ids = Vec::new();
	let mut field_tags = Vec::new();
	let mut field_data_types = Vec::new();
	let mut field_field_types = Vec::new();

	let mut field_value_ids = Vec::new();
	let mut field_value_field_ids = Vec::new();
	let mut field_values = Vec::new();

	for (tag, value) in &request.fields {
		// Convert the JSON value to a string representation
		let value_str = match value {
			serde_json::Value::String(s) => s.clone(),
			serde_json::Value::Number(n) => n.to_string(),
			serde_json::Value::Bool(b) => b.to_string(),
			serde_json::Value::Array(arr) => {
				// For arrays, we'll convert to a comma-separated string
				arr.iter()
					.map(|v| match v {
						serde_json::Value::String(s) => s.clone(),
						_ => serde_json::to_string(v).unwrap_or_else(|_| "null".to_string()),
					})
					.collect::<Vec<_>>()
					.join(",")
			}
			serde_json::Value::Object(_) => {
				// For objects, serialize to JSON string
				serde_json::to_string(value).unwrap_or_else(|_| "null".to_string())
			}
			serde_json::Value::Null => "null".to_string(),
		};

		// Skip empty fields (but allow "null" as a valid value)
		if value_str.trim().is_empty() && value_str != "null" {
			continue;
		}

		let field_id = Uuid::new_v4();
		field_ids.push(field_id);
		field_ad_ids.push(ad_id);
		field_tags.push(tag.clone());

		// Determine data type based on the original value
		let data_type = match value {
			serde_json::Value::String(_) => "string".to_string(),
			serde_json::Value::Number(n) => {
				if n.is_f64() {
					"float".to_string()
				} else {
					"integer".to_string()
				}
			}
			serde_json::Value::Bool(_) => "boolean".to_string(),
			serde_json::Value::Array(_) => "array".to_string(),
			serde_json::Value::Object(_) => "object".to_string(),
			serde_json::Value::Null => "null".to_string(),
		};
		field_data_types.push(data_type);
		field_field_types.push("attribute".to_string());

		let field_value_id = Uuid::new_v4();
		field_value_ids.push(field_value_id);
		field_value_field_ids.push(field_id);
		field_values.push(value_str);
	}

	// Batch insert fields if any exist
	if !field_ids.is_empty() {
		println!("Updating {} fields for ad ID: {}", field_ids.len(), ad_id);
		sqlx::query!(
			r#"
            INSERT INTO avito_ad_fields (field_id, ad_id, tag, data_type, field_type)
            SELECT * FROM UNNEST(
                $1::uuid[],
                $2::uuid[],
                $3::varchar[],
                $4::varchar[],
                $5::varchar[]
            )
            "#,
			&field_ids,
			&field_ad_ids,
			&field_tags,
			&field_data_types,
			&field_field_types,
		)
		.execute(&mut *tx)
		.await
		.map_err(|e| {
			ApiError::InternalServerError(format!("Failed to batch insert fields: {}", e))
		})?;
		println!(
			"{} fields updated successfully for ad ID: {}",
			field_ids.len(),
			ad_id
		);
	} else {
		println!("No fields to update for ad ID: {}", ad_id);
	}

	// Batch insert field values if any exist
	if !field_value_ids.is_empty() {
		println!(
			"Updating {} field values for ad ID: {}",
			field_value_ids.len(),
			ad_id
		);
		sqlx::query!(
			r#"
            INSERT INTO avito_ad_field_values (field_value_id, field_id, value)
            SELECT * FROM UNNEST(
                $1::uuid[],
                $2::uuid[],
                $3::varchar[]
            )
            "#,
			&field_value_ids,
			&field_value_field_ids,
			&field_values,
		)
		.execute(&mut *tx)
		.await
		.map_err(|e| {
			ApiError::InternalServerError(format!("Failed to batch insert field values: {}", e))
		})?;
		println!(
			"{} field values updated successfully for ad ID: {}",
			field_value_ids.len(),
			ad_id
		);
	} else {
		println!("No field values to update for ad ID: {}", ad_id);
	}

	// Commit transaction
	tx.commit().await.map_err(|e| {
		ApiError::InternalServerError(format!("Failed to commit transaction: {}", e))
	})?;

	Ok(HttpResponse::Ok().json(UpdateAdResponse {
		ad_id,
		message: "Ad updated successfully".to_string(),
	}))
}

// Batch update function that can update multiple ads at once
#[derive(Debug, Deserialize)]
pub struct BatchUpdateAdRequest {
	pub updates: Vec<UpdateAdRequest>,
}

#[derive(Debug, Serialize)]
pub struct BatchUpdateAdResponse {
	pub updated_ads: Vec<Uuid>,
	pub message: String,
}

#[post("/avito/batch-update-ads")]
pub async fn avito_batch_update_ads(
	data: web::Data<AppState>,
	request: web::Json<BatchUpdateAdRequest>,
	_: JwtMiddleware,
) -> Result<HttpResponse, ApiError> {
	let mut updated_ads = Vec::new();

	for update_request in request.updates.iter() {
		// Process each ad update in the batch
		let ad_id = update_request.ad_id;
		let account_id = update_request.account_id.unwrap_or_else(|| {
			// Default account_id if not provided - you might want to get this from JWT claims instead
			Uuid::parse_str("2acc3808-15f1-4abb-b15e-c7f4780a87da").unwrap()
		});

		// Start transaction for each ad update
		let mut tx = data.db.begin().await.map_err(|e| {
			ApiError::InternalServerError(format!("Failed to start transaction: {}", e))
		})?;

		// Verify that the ad exists and belongs to the correct account
		let ad_exists = sqlx::query!(
			r#"
            SELECT a.ad_id
            FROM avito_ads a
            JOIN avito_feeds f ON a.feed_id = f.feed_id
            WHERE a.ad_id = $1 AND f.account_id = $2
            "#,
			ad_id,
			account_id
		)
		.fetch_optional(&mut *tx)
		.await
		.map_err(|e| {
			ApiError::InternalServerError(format!("Failed to check if ad exists: {}", e))
		})?;

		if ad_exists.is_none() {
			// Continue with the next ad instead of failing the entire batch
			continue;
		}

		// Delete existing field values and fields for this ad
		sqlx::query!(
			r#"
            DELETE FROM avito_ad_field_values
            WHERE field_id IN (
                SELECT field_id
                FROM avito_ad_fields
                WHERE ad_id = $1
            )
            "#,
			ad_id
		)
		.execute(&mut *tx)
		.await
		.map_err(|e| {
			ApiError::InternalServerError(format!("Failed to delete field values: {}", e))
		})?;

		sqlx::query!(
			r#"
            DELETE FROM avito_ad_fields
            WHERE ad_id = $1
            "#,
			ad_id
		)
		.execute(&mut *tx)
		.await
		.map_err(|e| ApiError::InternalServerError(format!("Failed to delete fields: {}", e)))?;

		// Process each field in the request for batch insert
		let mut field_ids = Vec::new();
		let mut field_ad_ids = Vec::new();
		let mut field_tags = Vec::new();
		let mut field_data_types = Vec::new();
		let mut field_field_types = Vec::new();

		let mut field_value_ids = Vec::new();
		let mut field_value_field_ids = Vec::new();
		let mut field_values = Vec::new();

		for (tag, value) in &update_request.fields {
			// Convert the JSON value to a string representation
			let value_str = match value {
				serde_json::Value::String(s) => s.clone(),
				serde_json::Value::Number(n) => n.to_string(),
				serde_json::Value::Bool(b) => b.to_string(),
				serde_json::Value::Array(arr) => {
					// For arrays, we'll convert to a comma-separated string
					arr.iter()
						.map(|v| match v {
							serde_json::Value::String(s) => s.clone(),
							_ => serde_json::to_string(v).unwrap_or_else(|_| "null".to_string()),
						})
						.collect::<Vec<_>>()
						.join(",")
				}
				serde_json::Value::Object(_) => {
					// For objects, serialize to JSON string
					serde_json::to_string(value).unwrap_or_else(|_| "null".to_string())
				}
				serde_json::Value::Null => "null".to_string(),
			};

			// Skip empty fields (but allow "null" as a valid value)
			if value_str.trim().is_empty() && value_str != "null" {
				continue;
			}

			let field_id = Uuid::new_v4();
			field_ids.push(field_id);
			field_ad_ids.push(ad_id);
			field_tags.push(tag.clone());

			// Determine data type based on the original value
			let data_type = match value {
				serde_json::Value::String(_) => "string".to_string(),
				serde_json::Value::Number(n) => {
					if n.is_f64() {
						"float".to_string()
					} else {
						"integer".to_string()
					}
				}
				serde_json::Value::Bool(_) => "boolean".to_string(),
				serde_json::Value::Array(_) => "array".to_string(),
				serde_json::Value::Object(_) => "object".to_string(),
				serde_json::Value::Null => "null".to_string(),
			};
			field_data_types.push(data_type);
			field_field_types.push("attribute".to_string());

			let field_value_id = Uuid::new_v4();
			field_value_ids.push(field_value_id);
			field_value_field_ids.push(field_id);
			field_values.push(value_str);
		}

		// Batch insert fields if any exist
		if !field_ids.is_empty() {
			sqlx::query!(
				r#"
                INSERT INTO avito_ad_fields (field_id, ad_id, tag, data_type, field_type)
                SELECT * FROM UNNEST(
                    $1::uuid[],
                    $2::uuid[],
                    $3::varchar[],
                    $4::varchar[],
                    $5::varchar[]
                )
                "#,
				&field_ids,
				&field_ad_ids,
				&field_tags,
				&field_data_types,
				&field_field_types,
			)
			.execute(&mut *tx)
			.await
			.map_err(|e| {
				ApiError::InternalServerError(format!("Failed to batch insert fields: {}", e))
			})?;
		}

		// Batch insert field values if any exist
		if !field_value_ids.is_empty() {
			sqlx::query!(
				r#"
                INSERT INTO avito_ad_field_values (field_value_id, field_id, value)
                SELECT * FROM UNNEST(
                    $1::uuid[],
                    $2::uuid[],
                    $3::varchar[]
                )
                "#,
				&field_value_ids,
				&field_value_field_ids,
				&field_values,
			)
			.execute(&mut *tx)
			.await
			.map_err(|e| {
				ApiError::InternalServerError(format!("Failed to batch insert field values: {}", e))
			})?;
		}

		// Commit transaction for this ad
		tx.commit().await.map_err(|e| {
			ApiError::InternalServerError(format!("Failed to commit transaction: {}", e))
		})?;

		updated_ads.push(ad_id);
	}

	Ok(HttpResponse::Ok().json(BatchUpdateAdResponse {
		updated_ads: updated_ads.clone(),
		message: format!("Successfully updated {} ads", updated_ads.len()),
	}))
}
