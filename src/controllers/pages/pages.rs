use crate::{
	models::{Count, FilterOptions, Page, PageBlock, PageBlockSection},
	AppState,
};
use actix_web::{
	get,
	web::{self, Path},
	HttpResponse, Responder,
};
use serde_json::json;
use uuid::Uuid;

use crate::utils::filter_page_record;

// #[get("/pages/{id}")]
// async fn get_pages_handler(
// 	path: Path<Uuid>,
// 	opts: web::Query<FilterOptions>,
// 	data: web::Data<AppState>,
// 	// _: jwt_auth::JwtMiddleware,
// ) -> impl Responder {
// 	let firm_id = &path.into_inner();
// 	let limit = opts.limit.unwrap_or(10);
// 	let offset = (opts.page.unwrap_or(1) - 1) * limit;
// 	let table = String::from("pages_items");

// 	let categories_query_result = sqlx::query_as!(
// 		PriceCategory,
// 		"SELECT * FROM pages_categories WHERE firm_id = $1 ORDER by created_ts LIMIT $2 OFFSET $3",
// 		firm_id,
// 		limit as i32,
// 		offset as i32
// 	)
// 	.fetch_all(&data.db)
// 	.await;

// 	let query_result = sqlx::query_as!(
// 		PriceItem,
// 		"SELECT * FROM pages_items WHERE firm_id = $1 ORDER by created_ts LIMIT $2 OFFSET $3",
// 		firm_id,
// 		limit as i32,
// 		offset as i32
// 	)
// 	.fetch_all(&data.db)
// 	.await;

// 	if query_result.is_err() {
// 		let message = "Что-то пошло не так во время чтения пользователей";
// 		return HttpResponse::InternalServerError()
// 			.json(json!({"status": "error","message": message}));
// 	}

// 	if categories_query_result.is_err() {
// 		let message = "Что-то пошло не так во время чтения категорий цен";
// 		return HttpResponse::InternalServerError()
// 			.json(json!({"status": "error","message": message}));
// 	}

// 	let pages_count = Count::count(&data.db, table).await.unwrap_or(0);

// 	let pages_categories = categories_query_result.unwrap();
// 	let pages_items = query_result.unwrap();

// 	let json_response = json!({
// 		"status":  "success",
// 		"data": json!({
// 			"pages_categories": &pages_categories.into_iter().map(|price| filter_price_category_record(&price)).collect::<Vec<FilteredPriceCategory>>(),
// 			"pages_items": &pages_items.into_iter().map(|price| filter_price_record(&price)).collect::<Vec<FilteredPriceItem>>(),
// 			"pages_count": &pages_count
// 		})
// 	});

// 	HttpResponse::Ok().json(json_response)
// }

#[get("/page_by_url/{id}")]
async fn get_page_by_url_handler(
	path: Path<String>,
	// opts: web::Query<FilterOptions>,
	data: web::Data<AppState>,
	// _: jwt_auth::JwtMiddleware,
) -> impl Responder {
	let page_url = &path.into_inner();
	let page_query_result = Page::get_page_by_url(&data.db, &page_url).await;
	let page_message = "Что-то пошло не так во время чтения get_page_by_url";
	if page_query_result.is_err() {
		return HttpResponse::InternalServerError()
			.json(json!({"status": "error","message": &page_message}));
	}
	let page = page_query_result.expect(&page_message);

	let page_block_query_result =
		PageBlock::get_page_blocks_by_page_id(&data.db, &page.page_id).await;
	if page_block_query_result.is_err() {
		return HttpResponse::InternalServerError()
			.json(json!({"status": "error","message": &page_message}));
	}
	let page_blocks = page_block_query_result.expect(&page_message);

	let mut page_blocks_sections = Vec::<PageBlockSection>::new();

	for page_block in page_blocks.iter() {
		let section_query_result = PageBlockSection::get_page_block_sections_by_block_id(
			&data.db,
			&page_block.page_block_id,
		)
		.await;
		if section_query_result.is_err() {
			return HttpResponse::InternalServerError()
				.json(json!({"status": "error","message": &page_message}));
		}
		for section in section_query_result.expect(&page_message) {
			page_blocks_sections.push(section);
		}
	}

	let json_response = json!({
		"status":  "success",
		"data": json!({
			"page": filter_page_record(&page),
			"blocks": page_blocks,
			"sections": page_blocks_sections
		})
	});

	HttpResponse::Ok().json(json_response)
}

#[get("/pages")]
async fn get_pages_handler(
	// opts: web::Query<FilterOptions>,
	data: web::Data<AppState>,
	// _: jwt_auth::JwtMiddleware,
) -> impl Responder {
	let pages_query_result = Page::get_pages(&data.db).await;
	let page_message = "Что-то пошло не так во время чтения get_page_by_url";
	if pages_query_result.is_err() {
		return HttpResponse::InternalServerError()
			.json(json!({"status": "error","message": &page_message}));
	}
	let pages = pages_query_result.expect(&page_message);

	let json_response = json!({
		"status":  "success",
		"data": json!({
			"pages": pages,
		})
	});

	HttpResponse::Ok().json(json_response)
}

#[get("/pages_by_firm/{id}")]
async fn get_pages_by_firm_handler(
	path: Path<Uuid>,
	opts: web::Query<FilterOptions>,
	data: web::Data<AppState>,
	// _: jwt_auth::JwtMiddleware,
) -> impl Responder {
	let firm_id = &path.into_inner();
	let limit = opts.limit.unwrap_or(10);
	let offset = (opts.page.unwrap_or(1) - 1) * limit;

	let pages_query_result =
		Page::get_pages_by_firm(&data.db, &firm_id, limit as i64, offset as i64).await;
	let page_message = "Что-то пошло не так во время чтения /pages_by_firm/{id}";
	if pages_query_result.is_err() {
		return HttpResponse::InternalServerError()
			.json(json!({"status": "error","message": &page_message}));
	}
	let pages = pages_query_result.expect(&page_message);

	let count_query_result = sqlx::query_as!(
		Count,
		"SELECT count(*) AS count FROM pages WHERE firm_id = $1",
		&firm_id
	)
	.fetch_one(&data.db)
	.await;

	let pages_count_message = "Что-то пошло не так во время подсчета pages";
	if count_query_result.is_err() {
		return HttpResponse::InternalServerError()
			.json(json!({"status": "error","message": &pages_count_message}));
	}
	let pages_count = count_query_result.expect(&pages_count_message);

	let json_response = json!({
		"status":  "success",
		"data": json!({
			"pages": pages,
			"pages_count": &pages_count.count.unwrap()
		})
	});

	HttpResponse::Ok().json(json_response)
}
