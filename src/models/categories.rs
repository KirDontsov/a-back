use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct Category {
	pub category_id: uuid::Uuid,
	pub name: Option<String>,
	pub abbreviation: Option<String>,
	pub single_name: Option<String>,
	pub rod_name: Option<String>,
	pub pred_name: Option<String>,
	pub vin_name: Option<String>,
	pub order_number: Option<String>,
	pub is_active: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct SaveCategory {
	pub name: String,
	pub abbreviation: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct FilteredCategory {
	pub category_id: String,
	pub name: Option<String>,
	pub abbreviation: Option<String>,
	pub single_name: Option<String>,
	pub rod_name: Option<String>,
	pub pred_name: Option<String>,
	pub vin_name: Option<String>,
	pub order_number: Option<String>,
	pub is_active: Option<String>,
}
