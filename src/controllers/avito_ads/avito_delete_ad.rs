use crate::{jwt_auth::JwtMiddleware, models::ApiError, AppState};
use actix_web::{
	post,
	web::{self},
	HttpResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct DeleteAdRequest {
	pub ad_id: Uuid,
	pub account_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct DeleteAdResponse {
	pub ad_id: Uuid,
	pub message: String,
}

#[post("/avito/delete-ad")]
pub async fn avito_delete_ad(
	data: web::Data<AppState>,
	request: web::Json<DeleteAdRequest>,
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

	// Delete field values for this ad's fields
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

	// Delete fields for this ad
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

	// Delete the ad itself
	sqlx::query!(
		r#"
        DELETE FROM avito_ads
        WHERE ad_id = $1
        "#,
		ad_id
	)
	.execute(&mut *tx)
	.await
	.map_err(|e| ApiError::InternalServerError(format!("Failed to delete ad: {}", e)))?;

	// Commit transaction
	tx.commit().await.map_err(|e| {
		ApiError::InternalServerError(format!("Failed to commit transaction: {}", e))
	})?;

	Ok(HttpResponse::Ok().json(DeleteAdResponse {
		ad_id,
		message: "Ad deleted successfully".to_string(),
	}))
}

// Delete multiple ads at once
#[derive(Debug, Deserialize)]
pub struct BatchDeleteAdRequest {
	pub ad_ids: Vec<Uuid>,
	pub account_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct BatchDeleteAdResponse {
	pub deleted_ads: Vec<Uuid>,
	pub message: String,
}

#[post("/avito/batch-delete-ads")]
pub async fn avito_batch_delete_ads(
	data: web::Data<AppState>,
	request: web::Json<BatchDeleteAdRequest>,
	_: JwtMiddleware,
) -> Result<HttpResponse, ApiError> {
	let ad_ids = &request.ad_ids;
	let account_id = request.account_id.unwrap_or_else(|| {
		// Default account_id if not provided
		Uuid::parse_str("2acc3808-15f1-4abb-b15e-c7f4780a87da").unwrap()
	});

	if ad_ids.is_empty() {
		return Ok(HttpResponse::Ok().json(BatchDeleteAdResponse {
			deleted_ads: vec![],
			message: "No ads to delete".to_string(),
		}));
	}

	// Start transaction
	let mut tx = data.db.begin().await.map_err(|e| {
		ApiError::InternalServerError(format!("Failed to start transaction: {}", e))
	})?;

	// Verify that the ads exist and belong to the correct account
	let existing_ads = sqlx::query!(
		r#"
        SELECT a.ad_id
        FROM avito_ads a
        JOIN avito_feeds f ON a.feed_id = f.feed_id
        WHERE a.ad_id = ANY($1) AND f.account_id = $2
        "#,
		&ad_ids[..],
		account_id
	)
	.fetch_all(&mut *tx)
	.await
	.map_err(|e| ApiError::InternalServerError(format!("Failed to check if ads exist: {}", e)))?;

	let existing_ad_ids: Vec<Uuid> = existing_ads.iter().map(|ad| ad.ad_id).collect();

	if existing_ad_ids.is_empty() {
		return Ok(HttpResponse::Ok().json(BatchDeleteAdResponse {
			deleted_ads: vec![],
			message: "No valid ads found to delete".to_string(),
		}));
	}

	// Delete field values for these ads' fields
	sqlx::query!(
		r#"
        DELETE FROM avito_ad_field_values
        WHERE field_id IN (
            SELECT field_id
            FROM avito_ad_fields
            WHERE ad_id = ANY($1)
        )
        "#,
		&existing_ad_ids[..]
	)
	.execute(&mut *tx)
	.await
	.map_err(|e| ApiError::InternalServerError(format!("Failed to delete field values: {}", e)))?;

	// Delete fields for these ads
	sqlx::query!(
		r#"
        DELETE FROM avito_ad_fields
        WHERE ad_id = ANY($1)
        "#,
		&existing_ad_ids[..]
	)
	.execute(&mut *tx)
	.await
	.map_err(|e| ApiError::InternalServerError(format!("Failed to delete fields: {}", e)))?;

	// Delete the ads themselves
	sqlx::query!(
		r#"
        DELETE FROM avito_ads
        WHERE ad_id = ANY($1)
        "#,
		&existing_ad_ids[..]
	)
	.execute(&mut *tx)
	.await
	.map_err(|e| ApiError::InternalServerError(format!("Failed to delete ads: {}", e)))?;

	// Commit transaction
	tx.commit().await.map_err(|e| {
		ApiError::InternalServerError(format!("Failed to commit transaction: {}", e))
	})?;

	Ok(HttpResponse::Ok().json(BatchDeleteAdResponse {
		deleted_ads: existing_ad_ids.clone(),
		message: format!("Successfully deleted {} ads", existing_ad_ids.len()),
	}))
}
