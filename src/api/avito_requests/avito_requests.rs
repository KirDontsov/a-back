use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::{api::CustomError, models::AvitoRequest};

impl AvitoRequest {
	pub async fn get_avito_requests_by_user(
		db: &Pool<Postgres>,
		user_id: &Uuid,
		limit: i64,
		offset: i64,
	) -> Result<Vec<Self>, CustomError> {
		let avito_requests_query_result = sqlx::query_as!(
			AvitoRequest,
			"SELECT * FROM avito_requests WHERE user_id = $1 ORDER by created_ts LIMIT $2 OFFSET $3",
			&user_id,
			&limit,
			&offset
		)
		.fetch_all(db)
		.await;

		if avito_requests_query_result.is_err() {
			println!("Что-то пошло не так во время запроса get_avito_requests");
		}

		Ok(avito_requests_query_result.unwrap_or(Vec::new()))
	}
}
