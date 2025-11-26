use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct Page {
	pub page_id: Uuid,
	pub firm_id: Option<Uuid>,
	pub page_category_id: Option<Uuid>,
	pub user_id: Option<Uuid>,
	pub url: Option<String>,
	pub prompt_value: Option<String>,
	pub oai_value: Option<String>,
	pub page_photo: Option<String>,
	#[serde(rename = "createdTs")]
	pub created_ts: Option<DateTime<Utc>>,
	#[serde(rename = "updatedTs")]
	pub updated_ts: Option<DateTime<Utc>>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct PageCategory {
	pub page_category_id: Uuid,
	pub page_id: Option<Uuid>,
	pub name: Option<String>,
	pub abbreviation: Option<String>,
	#[serde(rename = "createdTs")]
	pub created_ts: Option<DateTime<Utc>>,
	#[serde(rename = "updatedTs")]
	pub updated_ts: Option<DateTime<Utc>>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct FilteredPage {
	pub page_id: String,
	pub firm_id: String,
	pub page_category_id: String,
	pub user_id: String,
	pub url: Option<String>,
	pub page_photo: Option<String>,
	pub oai_value: Option<String>,
	#[serde(rename = "createdTs")]
	pub created_ts: Option<DateTime<Utc>>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct PageBlock {
	pub page_block_id: Uuid,
	pub page_id: Option<Uuid>,
	pub page_block_title: Option<String>,
	pub page_block_subtitle: Option<String>,
	pub page_block_type: Option<i16>,
	pub page_block_order: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct PageBlockSection {
	pub page_block_section_id: Uuid,
	pub page_block_id: Option<Uuid>,
	pub page_block_section_order: Option<String>,
	pub title: Option<String>,
	pub subtitle: Option<String>,
	pub text: Option<String>,
	pub url: Option<String>,
	pub photo: Option<String>,
}
