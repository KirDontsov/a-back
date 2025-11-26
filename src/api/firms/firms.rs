use sqlx::{Pool, Postgres};
use urlencoding::encode;
use uuid::Uuid;

use crate::{
	api::CustomError,
	models::{Firm, FirmForMap},
};

impl Firm {
	pub async fn get_firm_by_city_category(
		db: &Pool<Postgres>,
		table_name: String,
		city_id: Uuid,
		category_id: Uuid,
		n: i64,
	) -> Result<Self, CustomError> {
		let sql = format!(
			"
			SELECT * FROM {}
			WHERE city_id = '{}' AND category_id = '{}'
			ORDER BY created_ts DESC
			LIMIT 1 OFFSET '{}';
			",
			&table_name, &city_id, &category_id, &n,
		);
		let firm_query_result = sqlx::query_as::<_, Firm>(&sql).fetch_one(db).await;

		let message = "Что-то пошло не так во время запроса get_firm_by_city_category";

		if firm_query_result.is_err() {
			println!("{}", &message);
		}

		Ok(firm_query_result.expect(&message))
	}

	pub async fn get_firm_with_empty_field(
		db: &Pool<Postgres>,
		table_name: String,
		field_name: String,
		n: i64,
	) -> Result<Self, CustomError> {
		let sql = format!(
			"
			SELECT * FROM {}
			WHERE {} = '' OR {} IS NULL
			ORDER BY rating DESC
			LIMIT 1 OFFSET '{}';
			",
			&table_name, &field_name, &field_name, &n,
		);
		let firm_query_result = sqlx::query_as::<_, Firm>(&sql).fetch_one(db).await;

		if firm_query_result.is_err() {
			println!("Что-то пошло не так во время запроса firm {}", &table_name);
		}

		Ok(firm_query_result.unwrap())
	}

	/// GET фирма по url
	pub async fn get_firm_by_url(db: &Pool<Postgres>, url: &String) -> Result<Self, CustomError> {
		let firm_query_result = sqlx::query_as::<_, Firm>("SELECT * FROM firms WHERE url = $1")
			.bind(encode(url).to_string())
			.fetch_one(db)
			.await;

		let message = "Что-то пошло не так во время запроса get_firm_by_url";

		match firm_query_result {
			Ok(x) => Ok(x),
			Err(_) => Err(CustomError::InternalError(message.to_string())),
		}
	}

	pub async fn get_firms_by_city_catagory(
		db: &Pool<Postgres>,
		city_id: Uuid,
		category_id: Uuid,
		limit: i32,
		offset: i32,
	) -> Result<Vec<Self>, CustomError> {
		let query_result = sqlx::query_as::<_, Firm>(
			"SELECT * FROM firms
			WHERE city_id = $1
			AND category_id = $2
			ORDER BY rating DESC, reviews_count DESC
		 	LIMIT $3 OFFSET $4",
		)
		.bind(city_id)
		.bind(category_id)
		.bind(limit)
		.bind(offset)
		.fetch_all(db)
		.await;

		let message = "Что-то пошло не так во время запроса firm";

		if query_result.is_err() {
			println!("{}", &message);
		}

		Ok(query_result.expect(&message))
	}

	pub async fn get_firms_by_city_catagory_for_map(
		db: &Pool<Postgres>,
		city_id: Uuid,
		category_id: Uuid,
	) -> Result<Vec<FirmForMap>, CustomError> {
		let query_result = sqlx::query_as::<_, FirmForMap>(
			"SELECT name, address, coords, url FROM firms
			WHERE city_id = $1
			AND category_id = $2
			ORDER BY rating DESC, reviews_count DESC",
		)
		.bind(city_id)
		.bind(category_id)
		.fetch_all(db)
		.await;

		let message = "Что-то пошло не так во время запроса firm";

		if query_result.is_err() {
			println!("{}", &message);
		}

		Ok(query_result.expect(&message))
	}

	// TODO: доделать update_firm
	// pub async fn update_firm(db: &Pool<Postgres>, firm: SaveFirm) -> Result<Self, CustomError> {
	// 	let firm_query_result = sqlx::query_as!(
	// 		Firm,
	// 		r#"UPDATE firm SET firm_id = $1 WHERE firm_id = $2 RETURNING *"#,
	// 		(&firm.value.clone().parse::<i64>().unwrap() + 1).to_string(),
	// 		firm.firm_id,
	// 	)
	// 	.fetch_one(db)
	// 	.await;

	// 	if firm_query_result.is_err() {
	// 		println!("Что-то пошло не так во время подсчета фирм");
	// 	}

	// 	Ok(firm_query_result.unwrap())
	// }
}
