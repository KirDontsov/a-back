use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct Type {
	pub type_id: uuid::Uuid,
	pub name: Option<String>,
	pub abbreviation: Option<String>,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct SaveType {
	pub name: String,
	pub abbreviation: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct FilteredType {
	pub type_id: String,
	pub name: Option<String>,
	pub abbreviation: Option<String>,
}

