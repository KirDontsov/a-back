use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::{api::CustomError, models::Count};

impl Count {
	pub async fn count(db: &Pool<Postgres>, table_name: String) -> Result<i64, CustomError> {
		let sql = format!("SELECT count(*) AS count FROM {}", &table_name);
		let count_query_result = sqlx::query_as::<_, Count>(&sql).fetch_one(db).await;

		if count_query_result.is_err() {
			println!("Что-то пошло не так во время запроса count {}", &table_name);
		}

		let result = count_query_result.unwrap().count.unwrap();

		println!("Count result: {:?}", &result);

		Ok(result)
	}


	pub async fn count_firms_by_city_category(
		db: &Pool<Postgres>,
		table_name: String,
		city_id: Uuid,
		category_id: Uuid,
	) -> Result<i64, CustomError> {
		let sql = format!(
			"SELECT count(*) AS count FROM {} WHERE city_id = '{}' AND category_id = '{}'",
			&table_name, &city_id, &category_id
		);
		let count_query_result = sqlx::query_as::<_, Count>(&sql).fetch_one(db).await;

		if count_query_result.is_err() {
			println!("Что-то пошло не так во время запроса count {}", &table_name);
		}

		let result = count_query_result.unwrap().count.unwrap();

		println!("Count result: {:?}", &result);

		Ok(result)
	}

	pub async fn count_firms_with_empty_field(
		db: &Pool<Postgres>,
		table_name: String,
		field_name: String,
	) -> Result<i64, CustomError> {
		let sql = format!(
			"SELECT count(*) AS count FROM {} WHERE {} = '' OR {} IS NULL",
			&table_name, &field_name, &field_name
		);
		let count_query_result = sqlx::query_as::<_, Count>(&sql).fetch_one(db).await;

		if count_query_result.is_err() {
			println!("Что-то пошло не так во время запроса count {}", &table_name);
		}

		let result = count_query_result.unwrap().count.unwrap();

		println!("Count result: {:?}", &result);

		Ok(result)
	}

	pub async fn count_smth_by_firm(
		db: &Pool<Postgres>,
		table_name: String,
		firm_id: &Uuid,
	) -> Result<i64, CustomError> {
		let sql = format!(
			"SELECT count(*) AS count FROM {} WHERE firm_id = '{}'",
			&table_name, &firm_id,
		);
		let count_query_result = sqlx::query_as::<_, Count>(&sql).fetch_one(db).await;

		if count_query_result.is_err() {
			println!("Что-то пошло не так во время запроса count {}", &table_name);
		}

		let result = count_query_result.unwrap().count.unwrap();

		println!("Count result: {:?}", &result);

		Ok(result)
	}
}
