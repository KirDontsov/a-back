use chrono::prelude::*;
use serde::Serialize;

#[allow(non_snake_case)]
#[derive(Debug, Serialize)]
pub struct FilteredUser {
	pub id: String,
	pub name: String,
	pub email: String,
	pub role: String,
	pub photo: String,
	pub verified: bool,
	pub favourite: Vec<String>,
	pub createdAt: DateTime<Utc>,
	pub updatedAt: DateTime<Utc>,
}

#[derive(Serialize, Debug)]
pub struct UserData {
	pub user: FilteredUser,
}

#[derive(Serialize, Debug)]
pub struct UserResponse {
	pub status: String,
	pub data: UserData,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize)]
pub struct FilteredQuote {
	pub id: String,
	pub text: Option<String>,
	pub author: Option<String>,
	pub createdAt: DateTime<Utc>,
	pub updatedAt: DateTime<Utc>,
}

#[derive(Serialize, Debug)]
pub struct QuoteData {
	pub quote: FilteredQuote,
}

#[derive(Serialize, Debug)]
pub struct QuoteResponse {
	pub status: String,
	pub data: QuoteData,
}
