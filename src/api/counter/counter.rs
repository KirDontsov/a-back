use sqlx::{Pool, Postgres};

use crate::{
	api::CustomError,
	models::{Counter, SaveCounter},
};

impl Counter {
	pub async fn get_counter(db: &Pool<Postgres>, id: &String) -> Result<Self, CustomError> {
		let counter_query_result = sqlx::query_as!(
			Counter,
			"SELECT * FROM counter WHERE counter_id = $1;",
			uuid::Uuid::parse_str(&id.clone()).unwrap()
		)
		.fetch_one(db)
		.await;

		if counter_query_result.is_err() {
			println!("Что-то пошло не так во время подсчета фирм");
		}

		Ok(counter_query_result.unwrap())
	}

	pub async fn update_counter(
		db: &Pool<Postgres>,
		counter: SaveCounter,
	) -> Result<Self, CustomError> {
		let counter_query_result = sqlx::query_as!(
			Counter,
			r#"UPDATE counter SET value = $1 WHERE counter_id = $2 RETURNING *"#,
			(&counter.value.clone().parse::<i64>().unwrap() + 1).to_string(),
			counter.counter_id,
		)
		.fetch_one(db)
		.await;

		if counter_query_result.is_err() {
			println!("Что-то пошло не так во время подсчета фирм");
		}

		Ok(counter_query_result.unwrap())
	}
}
