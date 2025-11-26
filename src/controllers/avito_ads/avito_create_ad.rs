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
pub struct CreateAdRequest {
	pub fields: HashMap<String, serde_json::Value>,
	pub account_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct CreateAdResponse {
	pub ad_id: Uuid,
	pub message: String,
}

#[post("/avito/create-ad")]
pub async fn avito_create_ad(
	data: web::Data<AppState>,
	request: web::Json<CreateAdRequest>,
	_: JwtMiddleware,
) -> Result<HttpResponse, ApiError> {
	let ad_id = Uuid::new_v4();
	let account_id = request.account_id.unwrap_or_else(|| {
		// Default account_id if not provided - you might want to get this from JWT claims instead
		Uuid::parse_str("2acc3808-15f1-4abb-b15e-c7f4780a87da").unwrap()
	});

	dbg!(&account_id);

	// Start transaction
	let mut tx = data.db.begin().await.map_err(|e| {
		ApiError::InternalServerError(format!("Failed to start transaction: {}", e))
	})?;

	// Check if a feed with category "MANUAL_CREATE" already exists for this account
	let existing_feed = sqlx::query!(
		r#"
	       SELECT feed_id
	       FROM avito_feeds
	       WHERE account_id = $1 AND category = $2
	       ORDER BY created_ts DESC
	       LIMIT 1
	       "#,
		account_id,
		"MANUAL_CREATE"
	)
	.fetch_optional(&mut *tx)
	.await
	.map_err(|e| {
		ApiError::InternalServerError(format!("Failed to check for existing feed: {}", e))
	})?;

	let feed_id = if let Some(feed) = existing_feed {
		// Use existing feed
		println!("Using existing feed with ID: {}", feed.feed_id);
		feed.feed_id
	} else {
		// Create a new feed
		let new_feed_id = Uuid::new_v4();
		println!("Creating new feed with ID: {}", new_feed_id);
		sqlx::query!(
			r#"
	           INSERT INTO avito_feeds (feed_id, account_id, category)
	           VALUES ($1, $2, $3)
	           "#,
			new_feed_id,
			account_id,
			"MANUAL_CREATE"
		)
		.execute(&mut *tx)
		.await
		.map_err(|e| ApiError::InternalServerError(format!("Failed to create feed: {}", e)))?;
		println!("Feed created successfully with ID: {}", new_feed_id);
		new_feed_id
	};

	// Insert the ad record
	println!("Creating ad with ID: {} for feed ID: {}", ad_id, feed_id);
	sqlx::query!(
		r#"
        INSERT INTO avito_ads (ad_id, feed_id, is_active, status)
        VALUES ($1, $2, $3, $4)
        "#,
		ad_id,
		feed_id,
		true,      // is_active
		"created"  // status
	)
	.execute(&mut *tx)
	.await
	.map_err(|e| ApiError::InternalServerError(format!("Failed to create ad: {}", e)))?;
	println!("Ad created successfully with ID: {}", ad_id);

	// Process each field in the request
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
		println!("Creating {} fields for ad ID: {}", field_ids.len(), ad_id);
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
			"{} fields created successfully for ad ID: {}",
			field_ids.len(),
			ad_id
		);
	} else {
		println!("No fields to create for ad ID: {}", ad_id);
	}

	// Batch insert field values if any exist
	if !field_value_ids.is_empty() {
		println!(
			"Creating {} field values for ad ID: {}",
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
			"{} field values created successfully for ad ID: {}",
			field_value_ids.len(),
			ad_id
		);
	} else {
		println!("No field values to create for ad ID: {}", ad_id);
	}

	// Commit transaction
	tx.commit().await.map_err(|e| {
		ApiError::InternalServerError(format!("Failed to commit transaction: {}", e))
	})?;

	Ok(HttpResponse::Ok().json(CreateAdResponse {
		ad_id,
		message: "Ad created successfully".to_string(),
	}))
}

// SQL queries for reference:

/*
-- SELECT query to get the new created feed, ad inside it with all the fields and values
SELECT
	f.feed_id,
	f.account_id,
	f.category,
	f.created_ts as feed_created_ts,
	a.ad_id,
	a.parsed_id,
	a.avito_ad_id,
	a.is_active,
	a.status,
	a.created_ts as ad_created_ts,
	af.field_id,
	af.tag,
	af.data_type,
	af.field_type,
	af.created_ts as field_created_ts,
	afv.field_value_id,
	afv.value,
	afv.created_ts as value_created_ts
FROM avito_feeds f
LEFT JOIN avito_ads a ON f.feed_id = a.feed_id
LEFT JOIN avito_ad_fields af ON a.ad_id = af.ad_id
LEFT JOIN avito_ad_field_values afv ON af.field_id = afv.field_id
WHERE f.feed_id = $1  -- Replace $1 with the specific feed_id you want to query
ORDER BY a.created_ts DESC, af.created_ts DESC, afv.created_ts DESC;
*/

/*
-- DELETE query to remove all data by feed_id
-- This must be done in the correct order due to foreign key constraints
DELETE FROM avito_ad_field_values
WHERE field_id IN (
	SELECT af.field_id
	FROM avito_ads a
	JOIN avito_ad_fields af ON a.ad_id = af.ad_id
	WHERE a.feed_id = $1 -- Replace $1 with the specific feed_id
);

DELETE FROM avito_ad_fields
WHERE ad_id IN (
	SELECT ad_id
	FROM avito_ads
	WHERE feed_id = $1  -- Replace $1 with the specific feed_id
);

DELETE FROM avito_ads
WHERE feed_id = $1;  -- Replace $1 with the specific feed_id

DELETE FROM avito_feeds
WHERE feed_id = $1;  -- Replace $1 with the specific feed_id
*/
