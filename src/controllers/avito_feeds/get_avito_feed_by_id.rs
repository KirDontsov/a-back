use crate::{
    jwt_auth::JwtMiddleware,
    models::{AdResponse, ApiError, FeedQueryParams, FeedResponse, FieldResponse, FieldValueResponse},
    AppState,
};
use actix_web::{
    get,
    web::{self},
    HttpResponse,
};
use serde::Deserialize;
use uuid::Uuid;
use chrono;

use std::collections::HashMap;

// Path parameter for feed_id
#[derive(Deserialize)]
pub struct FeedIdPath {
    pub feed_id: Uuid,
}

#[get("/avito/feeds/{feed_id}")]
pub async fn get_avito_feed_by_id(
    path: web::Path<FeedIdPath>,
    opts: web::Query<FeedQueryParams>,
    data: web::Data<AppState>,
    _: JwtMiddleware,
) -> Result<HttpResponse, ApiError> {
    let feed_id = path.feed_id;
    let page = opts.page.unwrap_or(1);
    let limit = opts.limit.unwrap_or(10);
    let offset = (page - 1) * limit;

    // First, get the feed details
    let feed_row = sqlx::query!(
        r#"SELECT
            feed_id,
            account_id,
            COALESCE(category, 'Unknown') as category,
            created_ts as "created_ts: chrono::DateTime<chrono::Utc>"
           FROM avito_feeds
           WHERE feed_id = $1"#,
        feed_id
    )
    .fetch_optional(&data.db)
    .await
    .map_err(|e| ApiError::InternalServerError(format!("Failed to fetch feed: {}", e)))?;

    // If no feed exists, return empty response
    let feed = match feed_row {
        Some(feed) => feed,
        None => {
            return Ok(HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "data": null,
                "pagination": {
                    "page": page,
                    "limit": limit,
                    "total": 0,
                    "pages": 0
                }
            })));
        }
    };

    // Get total count of ads for this feed
    let count_row = sqlx::query!(
        r#"SELECT COUNT(*) as count FROM avito_ads WHERE feed_id = $1"#,
        feed_id
    )
    .fetch_one(&data.db)
    .await
    .map_err(|e| ApiError::InternalServerError(format!("Failed to fetch ad count: {}", e)))?;

    let total_ads = count_row.count.unwrap_or(0) as u32;

    // Get the paginated list of ads for this feed
    let ad_rows = sqlx::query!(
        r#"SELECT
            ad_id,
            COALESCE(avito_ad_id, '') as avito_ad_id,
            COALESCE(parsed_id, '') as parsed_id,
            COALESCE(is_active, true) as is_active,
            COALESCE(status, 'unknown') as status,
            created_ts as "created_ts: chrono::DateTime<chrono::Utc>"
        FROM avito_ads
        WHERE feed_id = $1
        ORDER BY created_ts DESC
        LIMIT $2 OFFSET $3"#,
        feed_id,
        limit as i64,
        offset as i64
    )
    .fetch_all(&data.db)
    .await
    .map_err(|e| ApiError::InternalServerError(format!("Failed to fetch ads: {}", e)))?;

    // Create ads map with empty fields initially and collect ad_ids
    let mut ads_map: HashMap<Uuid, AdResponse> = HashMap::new();
    let mut ad_ids: Vec<Uuid> = Vec::new();
    
    for row in &ad_rows {
        ads_map.insert(
            row.ad_id,
            AdResponse {
                ad_id: row.ad_id,
                avito_ad_id: row.avito_ad_id.clone().unwrap_or_default(),
                parsed_id: row.parsed_id.clone().unwrap_or_default(),
                is_active: row.is_active.unwrap_or(true),
                status: row.status.clone().unwrap_or_else(|| "unknown".to_string()),
                created_ts: row.created_ts.unwrap(),
                fields: Vec::new(),
            },
        );
        ad_ids.push(row.ad_id);
    }

    // Get all fields for these ads only if we have ads
    let mut fields_rows = Vec::new();
    let mut fields_map: HashMap<Uuid, FieldResponse> = HashMap::new();
    
    if !ad_ids.is_empty() {
        fields_rows = sqlx::query!(
            r#"SELECT
                field_id,
                ad_id,
                COALESCE(tag, '') as tag,
                COALESCE(data_type, 'string') as data_type,
                COALESCE(field_type, 'attribute') as field_type,
                created_ts as "created_ts: chrono::DateTime<chrono::Utc>"
            FROM avito_ad_fields
            WHERE ad_id = ANY($1)
            ORDER BY created_ts"#,
            &ad_ids
        )
        .fetch_all(&data.db)
        .await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to fetch fields: {}", e)))?;

        // Create fields map with empty values initially
        for row in &fields_rows {
            let field_response = FieldResponse {
                field_id: row.field_id,
                tag: row.tag.clone().unwrap_or_default(),
                data_type: row.data_type.clone().unwrap_or_else(|| "string".to_string()),
                field_type: row.field_type.clone().unwrap_or_else(|| "attribute".to_string()),
                created_ts: row.created_ts.unwrap(),
                values: Vec::new(),
            };
            fields_map.insert(row.field_id, field_response);
        }

        // Get all field values for these fields
        let mut field_ids: Vec<Uuid> = fields_rows.iter().map(|row| row.field_id).collect();
        
        if !field_ids.is_empty() {
            let field_values_rows = sqlx::query!(
                r#"SELECT
                    field_value_id,
                    field_id,
                    COALESCE(value, '') as value,
                    created_ts as "created_ts: chrono::DateTime<chrono::Utc>"
                FROM avito_ad_field_values
                WHERE field_id = ANY($1)
                ORDER BY created_ts"#,
                &field_ids
            )
            .fetch_all(&data.db)
            .await
            .map_err(|e| ApiError::InternalServerError(format!("Failed to fetch field values: {}", e)))?;

            // Add values to their respective fields
            for row in &field_values_rows {
                if let Some(field_id) = row.field_id {
                    if let Some(field) = fields_map.get_mut(&field_id) {
                        field.values.push(FieldValueResponse {
                            field_value_id: row.field_value_id,
                            value: row.value.clone().unwrap_or_default(),
                            created_ts: row.created_ts.unwrap(),
                        });
                    }
                }
            }
        }
    }

    // Attach fields to their respective ads
    for row in &fields_rows {
        if let Some(ad) = ads_map.get_mut(&row.ad_id) {
            if let Some(field) = fields_map.get(&row.field_id) {
                ad.fields.push(field.clone());
            }
        }
    }


    // Convert HashMap to Vec maintaining the original order from ad_rows
    let mut ads_vec: Vec<AdResponse> = Vec::new();
    for row in &ad_rows {
        if let Some(ad) = ads_map.get(&row.ad_id) {
            ads_vec.push(ad.clone());
        }
    }

    // Create the FeedResponse with the feed data and paginated ads
    let feed_response = FeedResponse {
        feed_id: feed.feed_id,
        account_id: feed.account_id,
        category: feed.category.unwrap_or_else(|| "Unknown".to_string()),
        created_ts: feed.created_ts.unwrap(),
        ads: ads_vec,
    };

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "data": feed_response,
        "pagination": {
            "page": page,
            "limit": limit,
            "total": total_ads,
            "pages": (total_ads as f64 / limit as f64).ceil() as u32
        }
    })))
}