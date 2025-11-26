use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct City {
	pub city_id: uuid::Uuid,
	pub name: Option<String>,
	pub abbreviation: Option<String>,
	pub coords: Option<String>,
	pub order_number: Option<String>,
	pub is_active: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct SaveCity {
	pub name: String,
	pub abbreviation: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct FilteredCity {
	pub city_id: String,
	pub name: Option<String>,
	pub abbreviation: Option<String>,
	pub coords: Option<String>,
	pub order_number: Option<String>,
	pub is_active: Option<String>,
}
