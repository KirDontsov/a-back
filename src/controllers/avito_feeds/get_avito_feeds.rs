use crate::{
    jwt_auth::JwtMiddleware,
    models::{ApiError, FeedQueryParams, FeedResponse},
    AppState,
};
use actix_web::{
    post,
    web::{self},
    HttpResponse,
};
use serde::Deserialize;
use uuid::Uuid;
use chrono;

// Structure for POST request body containing account_id
#[derive(Deserialize)]
pub struct AccountIdRequest {
    pub account_id: Uuid,
}

#[post("/avito/feeds")]
pub async fn get_avito_feeds(
    body: web::Json<AccountIdRequest>,
    opts: web::Query<FeedQueryParams>,
    data: web::Data<AppState>,
    _: JwtMiddleware,
) -> Result<HttpResponse, ApiError> {
    let account_id = body.account_id;
    let page = opts.page.unwrap_or(1);
    let limit = opts.limit.unwrap_or(10);
    let offset = (page - 1) * limit;

    // Fetch only the basic feed information
    let feed_rows = sqlx::query!(
        r#"
        SELECT feed_id, account_id, category, created_ts
        FROM avito_feeds
        WHERE account_id = $1
        ORDER BY created_ts DESC
        LIMIT $2 OFFSET $3
        "#,
        account_id,
        limit as i64,
        offset as i64
    )
    .fetch_all(&data.db)
    .await
    .map_err(|e| ApiError::InternalServerError(format!("Failed to fetch feeds: {}", e)))?;

    // Fetch total count of feeds for this account
    let count_row = sqlx::query!(
        r#"SELECT COUNT(*) as count FROM avito_feeds WHERE account_id = $1"#,
        account_id
    )
    .fetch_one(&data.db)
    .await
    .map_err(|e| ApiError::InternalServerError(format!("Failed to fetch feed count: {}", e)))?;

    let total_feeds = count_row.count.unwrap_or(0) as u32;

    // Convert to FeedResponse format
    let feeds_vec: Vec<FeedResponse> = feed_rows
        .into_iter()
        .map(|row| FeedResponse {
            feed_id: row.feed_id,
            account_id: row.account_id,
            category: row.category.unwrap_or_else(|| "unknown".to_string()),
            created_ts: row.created_ts.unwrap_or_else(|| chrono::Utc::now()),
            ads: Vec::new(), // Empty ads list since we're only returning basic feed info
        })
        .collect();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "data": feeds_vec,
        "pagination": {
            "page": page,
            "limit": limit,
            "total": total_feeds,
            "pages": (total_feeds as f64 / limit as f64).ceil() as u32
        }
    })))
}