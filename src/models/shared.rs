use serde::Deserialize;
use sqlx::FromRow;

#[derive(Deserialize, Debug)]
pub struct FilterOptions {
	pub page: Option<usize>,
	pub limit: Option<usize>,
}

#[derive(Deserialize, Debug, FromRow)]
pub struct Count {
	pub count: Option<i64>,
}

#[derive(Deserialize, Debug)]
pub struct FilterExtOptions {
	pub input: Option<String>,
	pub city_id: Option<String>,
	pub category_id: Option<String>,
	// pub type_id: Option<String>,
	pub page: Option<usize>,
	pub limit: Option<usize>,
}
