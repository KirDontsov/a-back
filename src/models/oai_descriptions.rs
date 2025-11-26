use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct OAIDescription {
	pub oai_description_id: uuid::Uuid,
	pub firm_id: uuid::Uuid,
	pub oai_description_value: Option<String>,
	#[serde(rename = "createdTs")]
	pub created_ts: Option<DateTime<Utc>>,
	#[serde(rename = "updatedTs")]
	pub updated_ts: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct FilteredOAIDescription {
	pub oai_description_id: String,
	pub firm_id: String,
	pub oai_description_value: Option<String>,
}
