use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct User {
	pub id: uuid::Uuid,
	pub name: String,
	pub email: String,
	pub password: String,
	pub role: String,
	pub photo: String,
	pub verified: bool,
	pub favourite: Option<Vec<String>>,
	#[serde(rename = "createdAt")]
	pub created_at: Option<DateTime<Utc>>,
	#[serde(rename = "updatedAt")]
	pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
	pub sub: String,
	pub iat: usize,
	pub exp: usize,
}

#[derive(Debug, Deserialize)]
pub struct RegisterUserSchema {
	pub name: String,
	pub email: String,
	pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginUserSchema {
	pub email: String,
	pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserSchema {
	pub name: String,
	pub email: String,
	pub role: String,
	pub verified: bool,
	pub favourite: Vec<String>,
	// #[serde(rename = "updatedAt")]
	// pub updated_at: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct Quote {
	pub id: uuid::Uuid,
	pub text: Option<String>,
	pub author: Option<String>,
	#[serde(rename = "createdAt")]
	pub created_at: Option<DateTime<Utc>>,
	#[serde(rename = "updatedAt")]
	pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Debug)]
pub struct FilterOptions {
	pub page: Option<usize>,
	pub limit: Option<usize>,
}
