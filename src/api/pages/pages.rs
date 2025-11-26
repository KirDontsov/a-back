use sqlx::{Pool, Postgres};
use urlencoding::encode;
use uuid::Uuid;

use crate::{
	api::CustomError,
	models::{Page, PageBlock, PageBlockSection},
};

impl Page {
	/// GET страница по url
	pub async fn get_page_by_url(db: &Pool<Postgres>, url: &String) -> Result<Self, CustomError> {
		let page_query_result = sqlx::query_as::<_, Page>("SELECT * FROM pages WHERE url = $1")
			.bind(encode(url).to_string())
			.fetch_one(db)
			.await;

		let message = "Что-то пошло не так во время запроса get_page_by_url";

		match page_query_result {
			Ok(x) => Ok(x),
			Err(_) => Err(CustomError::InternalError(message.to_string())),
		}
	}

	/// GET страница по url
	pub async fn get_pages(db: &Pool<Postgres>) -> Result<Vec<Self>, CustomError> {
		let pages_query_result = sqlx::query_as::<_, Page>(
			"SELECT * FROM pages WHERE firm_id IS NULL ORDER BY created_ts",
		)
		.fetch_all(db)
		.await;

		let message = "Что-то пошло не так во время запроса get_pages";

		match pages_query_result {
			Ok(x) => Ok(x),
			Err(_) => Err(CustomError::InternalError(message.to_string())),
		}
	}

	/// GET страница по url
	pub async fn get_pages_by_firm(
		db: &Pool<Postgres>,
		id: &Uuid,
		limit: i64,
		offset: i64,
	) -> Result<Vec<Self>, CustomError> {
		let pages_query_result = sqlx::query_as::<_, Page>(
			"SELECT * FROM pages WHERE firm_id = $1 ORDER BY created_ts LIMIT $2 OFFSET $3",
		)
		.bind(id)
		.bind(&limit)
		.bind(&offset)
		.fetch_all(db)
		.await;

		let message = "Что-то пошло не так во время запроса get_pages_by_firm";

		match pages_query_result {
			Ok(x) => Ok(x),
			Err(_) => Err(CustomError::InternalError(message.to_string())),
		}
	}
}

impl PageBlock {
	pub async fn get_page_blocks_by_page_id(
		db: &Pool<Postgres>,
		id: &Uuid,
	) -> Result<Vec<Self>, CustomError> {
		let page_block_query_result = sqlx::query_as::<_, PageBlock>(
			"SELECT * FROM pages_blocks WHERE page_id = $1 ORDER BY page_block_order",
		)
		.bind(id)
		.fetch_all(db)
		.await;

		let message = "Что-то пошло не так во время запроса get_page_blocks_by_page_id";

		match page_block_query_result {
			Ok(x) => Ok(x),
			Err(_) => Err(CustomError::InternalError(message.to_string())),
		}
	}
}

impl PageBlockSection {
	pub async fn get_page_block_sections_by_block_id(
		db: &Pool<Postgres>,
		id: &Uuid,
	) -> Result<Vec<Self>, CustomError> {
		let page_block_query_result = sqlx::query_as::<_, PageBlockSection>(
			"SELECT * FROM pages_blocks_sections WHERE page_block_id = $1 ORDER BY page_block_section_order",
		)
		.bind(id)
		.fetch_all(db)
		.await;

		let message = "Что-то пошло не так во время запроса get_page_block_sections_by_block_id";

		match page_block_query_result {
			Ok(x) => Ok(x),
			Err(_) => Err(CustomError::InternalError(message.to_string())),
		}
	}
}
