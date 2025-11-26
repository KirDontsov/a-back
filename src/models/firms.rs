use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::postgres::types::TsVector;
use uuid::Uuid;

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct TwoGisFirm {
	pub firm_id: Uuid,
	pub name: Option<String>,
	pub two_gis_firm_id: Option<String>,
	pub category_id: Option<String>,
	pub coords: Option<String>,
	#[serde(rename = "createdTs")]
	pub created_ts: Option<DateTime<Utc>>,
	#[serde(rename = "updatedTs")]
	pub updated_ts: Option<DateTime<Utc>>,
}

#[allow(non_snake_case)]
#[derive(Debug, sqlx::FromRow, sqlx::Type, Default)]
pub struct Firm {
	pub firm_id: Uuid,
	pub category_id: Uuid,
	pub type_id: Uuid,
	pub city_id: Uuid,
	pub two_gis_firm_id: Option<String>,
	pub name: Option<String>,
	pub description: Option<String>,
	pub address: Option<String>,
	pub floor: Option<String>,
	pub site: Option<String>,
	pub default_email: Option<String>,
	pub default_phone: Option<String>,
	pub url: Option<String>,
	pub rating: Option<String>,
	pub reviews_count: Option<String>,
	pub coords: Option<String>,
	pub title: Option<String>,
	pub ts: Option<TsVector>,
	pub created_ts: Option<DateTime<Utc>>,
	pub updated_ts: Option<DateTime<Utc>>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct SaveFirm {
	pub two_gis_firm_id: String,
	pub category_id: Uuid,
	pub type_id: Uuid,
	pub city_id: Uuid,
	pub name: String,
	pub address: String,
	pub coords: String,
	// pub floor: String,
	pub default_phone: String,
	pub site: String,
	// pub default_email: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct FilteredFirm {
	pub firm_id: String,
	pub two_gis_firm_id: Option<String>,
	pub category_id: String,
	pub city_id: String,
	pub name: Option<String>,
	pub description: Option<String>,
	pub address: Option<String>,
	pub site: Option<String>,
	pub rating: Option<String>,
	pub reviews_count: Option<String>,
	pub default_phone: Option<String>,
	pub default_email: Option<String>,
	pub url: Option<String>,
	pub coords: Option<String>,
	pub floor: Option<String>,
	pub title: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct UpdateFirmDesc {
	pub firm_id: Uuid,
	pub description: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone, Default)]
pub struct ExtFirmWithOaiDescription {
	pub firm_id: Uuid,
	pub city_id: Uuid,
	pub category_id: Uuid,
	pub name: Option<String>,
	pub address: Option<String>,
	pub site: Option<String>,
	pub default_phone: Option<String>,
	pub oai_description_value: Option<String>,
	pub description: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct ExtFilteredFirmWithOaiDescription {
	pub firm_id: String,
	pub category_id: String,
	pub name: Option<String>,
	pub address: Option<String>,
	pub site: Option<String>,
	pub default_phone: Option<String>,
	pub oai_description_value: Option<String>,
	pub description: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct UpdateFirmAddress {
	pub firm_id: Uuid,
	pub address: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct UpdateFirmRating {
	pub firm_id: Uuid,
	pub rating: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct FirmForMap {
	pub name: Option<String>,
	pub address: Option<String>,
	pub url: Option<String>,
	pub coords: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct FilteredFirmForMap {
	pub name: Option<String>,
	pub address: Option<String>,
	pub url: Option<String>,
	pub coords: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct UpdateFirmCoords {
	pub firm_id: Uuid,
	pub coords: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct UpdateFirm {
	pub firm_id: Uuid,
	pub category_id: Uuid,
	pub city_id: Uuid,
	pub name: String,
	pub description: String,
	pub address: String,
	pub floor: String,
	pub site: String,
	pub default_email: String,
	pub default_phone: String,
	pub url: String,
	pub rating: String,
	pub reviews_count: String,
	pub coords: String,
	pub title: String,
	pub created_ts: Option<DateTime<Utc>>,
	pub updated_ts: Option<DateTime<Utc>>,
}
