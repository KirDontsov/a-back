use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct PriceCategory {
	pub price_category_id: uuid::Uuid,
	pub firm_id: uuid::Uuid,
	pub value: Option<String>,
	pub name: Option<String>,
	#[serde(rename = "createdTs")]
	pub created_ts: Option<DateTime<Utc>>,
	#[serde(rename = "updatedTs")]
	pub updated_ts: Option<DateTime<Utc>>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct PriceItem {
	pub price_item_id: uuid::Uuid,
	pub firm_id: uuid::Uuid,
	pub price_category_id: uuid::Uuid,
	pub value: Option<String>,
	pub name: Option<String>,
	#[serde(rename = "createdTs")]
	pub created_ts: Option<DateTime<Utc>>,
	#[serde(rename = "updatedTs")]
	pub updated_ts: Option<DateTime<Utc>>,
}

// #[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
// pub struct SaveReview {
// 	pub firm_id: uuid::Uuid,
// 	pub two_gis_firm_id: String,
// 	pub author: String,
// 	pub date: String,
// 	pub text: String,
// 	// pub rating: String,
// }

#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct FilteredPriceItem {
	pub price_item_id: String,
	pub firm_id: String,
	pub price_category_id: String,
	pub name: Option<String>,
	pub value: Option<String>,
}

#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct FilteredPriceCategory {
	pub price_category_id: String,
	pub firm_id: String,
	pub name: Option<String>,
	pub value: Option<String>,
}

// #[allow(non_snake_case)]
// #[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
// pub struct OAIReview {
// 	pub oai_review_id: uuid::Uuid,
// 	pub firm_id: uuid::Uuid,
// 	pub text: Option<String>,
// 	#[serde(rename = "createdTs")]
// 	pub created_ts: Option<DateTime<Utc>>,
// 	#[serde(rename = "updatedTs")]
// 	pub updated_ts: Option<DateTime<Utc>>,
// }

// #[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
// pub struct SaveOAIReview {
// 	pub firm_id: uuid::Uuid,
// 	pub text: String,
// }

// #[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
// pub struct FilteredOAIReview {
// 	pub oai_review_id: String,
// 	pub firm_id: String,
// 	pub text: Option<String>,
// }
