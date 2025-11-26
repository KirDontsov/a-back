use crate::models::{Counter, SaveCounter};
use sqlx::{Pool, Postgres};
use std::env;

pub async fn get_counter(db: &Pool<Postgres>, id: &String) -> i64 {
	let counter_query_result = Counter::get_counter(db, id).await.unwrap();

	counter_query_result
		.value
		.clone()
		.unwrap()
		.parse::<i64>()
		.unwrap()
}

pub async fn update_counter(db: &Pool<Postgres>, id: &String, value: &String) -> Counter {
	let city_id = uuid::Uuid::parse_str(
		env::var("CRAWLER_CITY_ID")
			.expect("CRAWLER_CITY_ID not set")
			.as_str(),
	)
	.unwrap();
	let category_id = uuid::Uuid::parse_str(
		env::var("CRAWLER_CATEGORY_ID")
			.expect("CRAWLER_CATEGORY_ID not set")
			.as_str(),
	)
	.unwrap();

	let counter_query_result = Counter::update_counter(
		db,
		SaveCounter {
			counter_id: uuid::Uuid::parse_str(&id).unwrap(),
			value: value.clone(),
			city_id: city_id.to_string(),
			category_id: category_id.to_string(),
		},
	)
	.await
	.unwrap();

	counter_query_result
}
