use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::{api::CustomError, models::Review};

impl Review {
	pub async fn get_reviews(
		db: &Pool<Postgres>,
		firm_id: Uuid,
		limit: i64,
		offset: i64,
	) -> Result<Vec<Self>, CustomError> {
		let reviews_query_result = sqlx::query_as!(
			Review,
			"SELECT * FROM reviews WHERE firm_id = $1 ORDER by created_ts LIMIT $2 OFFSET $3",
			&firm_id,
			&limit,
			&offset
		)
		.fetch_all(db)
		.await;

		if reviews_query_result.is_err() {
			println!("Что-то пошло не так во время запроса get_reviews");
		}

		Ok(reviews_query_result.unwrap_or(Vec::new()))
	}

	pub async fn get_all_reviews(
		db: &Pool<Postgres>,
		firm_id: &Uuid,
	) -> Result<Vec<Self>, CustomError> {
		let reviews_query_result = sqlx::query_as!(
			Review,
			"SELECT * FROM reviews WHERE firm_id = $1 ORDER by created_ts",
			&firm_id
		)
		.fetch_all(db)
		.await;

		if reviews_query_result.is_err() {
			println!("Что-то пошло не так во время запроса get_all_reviews");
		}

		Ok(reviews_query_result.unwrap_or(Vec::new()))
	}

	pub async fn get_reviews_by_firm(
		db: &Pool<Postgres>,
		firm_id: &Uuid,
		limit: i64,
		offset: i64,
	) -> Result<Vec<Self>, CustomError> {
		let reviews_query_result = sqlx::query_as!(
			Review,
			"SELECT * FROM reviews WHERE firm_id = $1 ORDER by created_ts LIMIT $2 OFFSET $3",
			&firm_id,
			&limit,
			&offset
		)
		.fetch_all(db)
		.await;

		let message = "Что-то пошло не так во время запроса get_reviews_by_firm";

		match reviews_query_result {
			Ok(x) => Ok(x),
			Err(_) => Err(CustomError::InternalError(message.to_string())),
		}
	}
}
