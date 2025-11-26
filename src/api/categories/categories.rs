use sqlx::{Pool, Postgres};

use crate::{api::CustomError, models::Category};

impl Category {
	pub async fn get_category_by_abbr(
		db: &Pool<Postgres>,
		category_id: &String,
	) -> Result<Self, CustomError> {
		let category_query_result = sqlx::query_as::<_, Category>(
			"SELECT * FROM categories
			WHERE abbreviation = $1
			",
		)
		.bind(category_id)
		.fetch_one(db)
		.await;

		let message = "Что-то пошло не так во время запроса get_category_by_abbr";

		match category_query_result {
			Ok(x) => Ok(x),
			Err(_) => Err(CustomError::InternalError(message.to_string())),
		}
	}
}
