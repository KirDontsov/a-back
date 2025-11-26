use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvitoAccount {
	pub account_id: Uuid,
	pub user_id: String,
	pub client_id: String,
	pub avito_client_secret: String,
	pub avito_client_id: String,
	pub is_connected: Option<bool>,
	#[serde(rename = "createdTs")]
	pub created_ts: DateTime<Utc>,
	#[serde(rename = "updatedTs")]
	pub updated_ts: DateTime<Utc>,
}

// Database representation that handles nullable fields
#[derive(sqlx::FromRow)]
pub struct DbAvitoAccount {
	pub account_id: Uuid,
	pub user_id: Option<String>,
	pub client_id: Option<String>,
	pub avito_client_secret: Option<String>,
	pub avito_client_id: Option<String>,
	pub is_connected: Option<bool>,
	pub created_ts: Option<NaiveDateTime>,
	pub updated_ts: Option<NaiveDateTime>,
}

impl From<DbAvitoAccount> for AvitoAccount {
	fn from(db_account: DbAvitoAccount) -> Self {
		AvitoAccount {
			account_id: db_account.account_id,
			user_id: db_account.user_id.unwrap_or_default(),
			client_id: db_account.client_id.unwrap_or_default(),
			avito_client_secret: db_account.avito_client_secret.unwrap_or_default(),
			avito_client_id: db_account.avito_client_id.unwrap_or_default(),
			is_connected: db_account.is_connected,
			created_ts: db_account
				.created_ts
				.map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
				.unwrap_or_else(|| Utc::now()),
			updated_ts: db_account
				.updated_ts
				.map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
				.unwrap_or_else(|| Utc::now()),
		}
	}
}

#[derive(Debug, Deserialize)]
pub struct CreateAvitoAccountSchema {
	pub user_id: String,
	pub avito_client_secret: String,
	pub avito_client_id: String,
	pub is_connected: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAvitoAccountSchema {
	pub user_id: Option<String>,
	pub client_id: Option<String>,
	pub avito_client_secret: Option<String>,
	pub avito_client_id: Option<String>,
	pub is_connected: Option<bool>,
}
