use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct AvitoRequest {
	pub request_id: uuid::Uuid,
	pub user_id: uuid::Uuid,
	/// String that represents a search query
	pub request: Option<String>,
	/// String that represents a filter query
	pub city: Option<String>,
	/// String that represents a filter query
	pub coords: Option<String>,
	/// String that represents a filter query
	pub radius: Option<String>,
	/// String that represents a filter query
	pub district: Option<String>,
	#[serde(rename = "createdTs")]
	pub created_ts: Option<DateTime<Utc>>,
	#[serde(rename = "updatedTs")]
	pub updated_ts: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct SaveAvitoRequest {
	pub request: String,
	pub city: String,
	pub coords: String,
	pub radius: String,
	pub district: String,
}

#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct FilteredAvitoRequest {
	pub request_id: String,
	pub user_id: String,
	pub request: Option<String>,
	pub city: Option<String>,
	pub coords: Option<String>,
	pub radius: Option<String>,
	pub district: Option<String>,
	#[serde(rename = "createdTs")]
	pub created_ts: Option<DateTime<Utc>>,
}

#[derive(sqlx::FromRow, Debug, Deserialize, Serialize, Clone)]
pub struct AdRecord {
	pub ad_id: Option<uuid::Uuid>, // Auto-generated primary key
	pub my_ad: String,
	pub run_date: DateTime<Utc>,
	pub city_query: String,
	pub search_query: String,
	pub position: i32,
	pub views: String,
	pub views_today: String,
	pub promotion: String,
	pub delivery: String,
	pub ad_date: String,
	pub avito_ad_id: String,
	pub title: String,
	pub price: String,
	pub link: String,
	pub categories: String,
	pub seller_id: String,
	pub seller_name: String,
	pub seller_type: String,
	pub register_date: String,
	pub answer_time: String,
	pub rating: String,
	pub reviews_count: String,
	pub ads_count: String,
	pub closed_ads_count: String,
	pub photo_count: String,
	pub address: String,
	pub description: String,
	pub avito_request_id: uuid::Uuid,
	pub created_ts: Option<DateTime<Utc>>,
}
