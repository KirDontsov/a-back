use crate::{
	jwt_auth::JwtMiddleware,
	models::{AdResponse, ApiError},
	AppState,
};
use actix_web::{
	post,
	web::{self},
	HttpResponse,
};
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

// Path parameters for feed_id and ad_id
#[derive(Deserialize)]
pub struct FeedAdPath {
	pub feed_id: Uuid,
	pub ad_id: Uuid,
}

// Request body to accept account_id
#[derive(Deserialize)]
pub struct FeedAdRequest {
	pub account_id: Uuid,
}

#[post("/avito/feed/{feed_id}/ad/{ad_id}")]
pub async fn get_avito_feed_ad(
	path: web::Path<FeedAdPath>,
	req_body: web::Json<FeedAdRequest>,
	data: web::Data<AppState>,
	_: JwtMiddleware,
) -> Result<HttpResponse, ApiError> {
	let feed_id = path.feed_id;
	let ad_id = path.ad_id;
	let account_id = req_body.account_id;

	// First, verify that the feed belongs to the account
	let feed_row = sqlx::query!(
		r#"SELECT feed_id, account_id
           FROM avito_feeds
           WHERE feed_id = $1 AND account_id = $2"#,
		feed_id,
		account_id
	)
	.fetch_optional(&data.db)
	.await
	.map_err(|e| ApiError::InternalServerError(format!("Failed to fetch feed: {}", e)))?;

	// If no feed exists for this account, return error
	if feed_row.is_none() {
		return Ok(HttpResponse::NotFound().json(serde_json::json!({
			"status": "error",
			"message": "Feed not found or does not belong to the specified account"
		})));
	}

	// Get the specific ad for this feed
	let ad_row = sqlx::query!(
		r#"SELECT
            ad_id,
            COALESCE(avito_ad_id, '') as avito_ad_id,
            COALESCE(parsed_id, '') as parsed_id,
            COALESCE(is_active, true) as is_active,
            COALESCE(status, 'unknown') as status,
            created_ts as "created_ts: chrono::DateTime<chrono::Utc>"
        FROM avito_ads
        WHERE ad_id = $1 AND feed_id = $2"#,
		ad_id,
		feed_id
	)
	.fetch_optional(&data.db)
	.await
	.map_err(|e| ApiError::InternalServerError(format!("Failed to fetch ad: {}", e)))?;

	// If no ad exists, return empty response
	let ad = match ad_row {
		Some(ad) => ad,
		None => {
			return Ok(HttpResponse::NotFound().json(serde_json::json!({
				"status": "error",
				"message": "Ad not found in the specified feed"
			})));
		}
	};

	// Get all fields and their values for this ad in a single query using a join
	let fields_with_values = sqlx::query!(
		r#"SELECT
    		f.field_id,
    		f.ad_id,
    		COALESCE(f.tag, '') as tag,
    		COALESCE(f.data_type, 'string') as data_type,
    		COALESCE(f.field_type, 'attribute') as field_type,
    		f.created_ts as "created_ts: chrono::DateTime<chrono::Utc>",
    		v.field_value_id,
    		COALESCE(v.value, '') as value,
    		v.created_ts as "value_created_ts: chrono::DateTime<chrono::Utc>"
    	FROM avito_ad_fields f
    	LEFT JOIN avito_ad_field_values v ON f.field_id = v.field_id
    	WHERE f.ad_id = $1
    	ORDER BY f.created_ts, v.created_ts"#,
		ad.ad_id
	)
	.fetch_all(&data.db)
	.await
	.map_err(|e| {
		ApiError::InternalServerError(format!("Failed to fetch fields and values: {}", e))
	})?;

	// Group field values by field_id
	let mut field_map: HashMap<uuid::Uuid, crate::models::FieldResponse> = HashMap::new();

	for row in &fields_with_values {
		let field_id = row.field_id;
		if !field_map.contains_key(&field_id) {
			field_map.insert(
				field_id,
				crate::models::FieldResponse {
					field_id: row.field_id,
					tag: row.tag.clone().unwrap_or_default(),
					data_type: row
						.data_type
						.clone()
						.unwrap_or_else(|| "string".to_string()),
					field_type: row
						.field_type
						.clone()
						.unwrap_or_else(|| "attribute".to_string()),
					created_ts: row.created_ts.unwrap(),
					values: Vec::new(),
				},
			);
		}

		if let Some(field) = field_map.get_mut(&field_id) {
			field.values.push(crate::models::FieldValueResponse {
				field_value_id: row.field_value_id,
				value: row.value.clone().unwrap_or_default(),
				created_ts: row.value_created_ts.unwrap(),
			});
		}
	}

	let fields: Vec<crate::models::FieldResponse> = field_map.into_values().collect();

	// Create the AdResponse with the ad data and its fields
	let ad_response = AdResponse {
		ad_id: ad.ad_id,
		avito_ad_id: ad.avito_ad_id.clone().unwrap_or_default(),
		parsed_id: ad.parsed_id.clone().unwrap_or_default(),
		is_active: ad.is_active.unwrap_or(true),
		status: ad.status.clone().unwrap_or_else(|| "unknown".to_string()),
		created_ts: ad.created_ts.unwrap(),
		fields,
	};

	Ok(HttpResponse::Ok().json(serde_json::json!({
		"status": "success",
		"data": ad_response
	})))
}
