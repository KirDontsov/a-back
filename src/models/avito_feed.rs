use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use uuid::Uuid;

// Generic structure to hold any XML tag and its value
#[derive(Debug)]
pub struct XmlAd {
	pub id: String,
	pub fields: HashMap<String, String>,
}

// Query parameters for pagination
#[derive(Debug, Deserialize)]
pub struct FeedQueryParams {
	pub page: Option<u32>,
	pub limit: Option<u32>,
}

// Response structures
#[derive(Debug, Serialize, Clone)]
pub struct FeedResponse {
	pub feed_id: Uuid,
	pub account_id: Uuid,
	pub category: String,
	pub created_ts: chrono::DateTime<chrono::Utc>,
	pub ads: Vec<AdResponse>,
}

#[derive(Debug, Serialize, Clone)]
pub struct AdResponse {
	pub ad_id: Uuid,
	pub avito_ad_id: String,
	pub parsed_id: String,
	pub is_active: bool,
	pub status: String,
	pub created_ts: chrono::DateTime<chrono::Utc>,
	pub fields: Vec<FieldResponse>,
}

#[derive(Debug, Serialize, Clone)]
pub struct FieldResponse {
	pub field_id: Uuid,
	pub tag: String,
	pub data_type: String,
	pub field_type: String,
	pub created_ts: chrono::DateTime<chrono::Utc>,
	pub values: Vec<FieldValueResponse>,
}

#[derive(Debug, Serialize, Clone)]
pub struct FieldValueResponse {
	pub field_value_id: Uuid,
	pub value: String,
	pub created_ts: chrono::DateTime<chrono::Utc>,
}

// Database row structure
#[derive(Debug, FromRow)]
pub struct FeedJoinRow {
	pub feed_id: Uuid,
	pub account_id: Uuid,
	pub category: String,
}
