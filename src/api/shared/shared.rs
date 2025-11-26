use sqlx::{Pool, Postgres};

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
}
