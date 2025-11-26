use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct Review {
	pub review_id: uuid::Uuid,
	pub firm_id: uuid::Uuid,
	pub two_gis_firm_id: Option<String>,
	pub author: Option<String>,
	pub date: Option<String>,
	pub rating: Option<String>,
	pub text: Option<String>,
	pub parsed: Option<bool>,
	#[serde(rename = "createdTs")]
	pub created_ts: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct SaveReview {
	pub firm_id: uuid::Uuid,
	pub two_gis_firm_id: String,
	pub author: String,
	pub date: String,
	pub text: String,
	pub rating: String,
}

#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct AddReview {
	pub firm_id: uuid::Uuid,
	// pub two_gis_firm_id: String,
	pub author: String,
	// pub date: String,
	pub text: String,
	pub rating: String,
}

#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct FilteredReview {
	pub review_id: String,
	pub firm_id: String,
	pub two_gis_firm_id: Option<String>,
	pub author: Option<String>,
	pub date: Option<String>,
	pub text: Option<String>,
	pub rating: Option<String>,
}

#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct FilteredAddReview {
	pub review_id: String,
	pub firm_id: String,
	pub author: Option<String>,
	pub date: Option<String>,
	pub text: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct OAIReview {
	pub oai_review_id: uuid::Uuid,
	pub firm_id: uuid::Uuid,
	pub text: Option<String>,
	#[serde(rename = "createdTs")]
	pub created_ts: Option<DateTime<Utc>>,
	#[serde(rename = "updatedTs")]
	pub updated_ts: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct SaveOAIReview {
	pub firm_id: uuid::Uuid,
	pub text: String,
}

#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct FilteredOAIReview {
	pub oai_review_id: String,
	pub firm_id: String,
	pub text: Option<String>,
}
