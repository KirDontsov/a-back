use sqlx::{Pool, Postgres};

use crate::{api::CustomError, models::City};

impl City {
	pub async fn get_city_by_abbr(
		db: &Pool<Postgres>,
		city_id: &String,
	) -> Result<Self, CustomError> {
		let city_query_result = sqlx::query_as::<_, City>(
			"SELECT * FROM cities
			WHERE abbreviation = $1
			",
		)
		.bind(city_id)
		.fetch_one(db)
		.await;

		let message = "Что-то пошло не так во время запроса get_city_by_abbr";

		match city_query_result {
			Ok(x) => Ok(x),
			Err(_) => Err(CustomError::InternalError(message.to_string())),
		}
	}
}
