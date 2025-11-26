use crate::{
	models::{
		Count, FilterOptions, FilteredPriceCategory, FilteredPriceItem, Firm, PriceCategory,
		PriceItem,
	},
	AppState,
};
use actix_web::{
	get,
	web::{self, Path},
	HttpResponse, Responder,
};
use serde_json::json;
use uuid::Uuid;

use crate::utils::{filter_price_category_record, filter_price_record};

#[get("/prices/{id}")]
async fn get_prices_handler(
	path: Path<Uuid>,
	opts: web::Query<FilterOptions>,
	data: web::Data<AppState>,
	// _: jwt_auth::JwtMiddleware,
) -> impl Responder {
	let firm_id = &path.into_inner();
	let limit = opts.limit.unwrap_or(10);
	let offset = (opts.page.unwrap_or(1) - 1) * limit;
	let table = String::from("prices_items");

	let categories_query_result = sqlx::query_as!(
		PriceCategory,
		"SELECT * FROM prices_categories WHERE firm_id = $1 ORDER by created_ts LIMIT $2 OFFSET $3",
		firm_id,
		limit as i32,
		offset as i32
	)
	.fetch_all(&data.db)
	.await;

	let query_result = sqlx::query_as!(
		PriceItem,
		"SELECT * FROM prices_items WHERE firm_id = $1 ORDER by created_ts LIMIT $2 OFFSET $3",
		firm_id,
		limit as i32,
		offset as i32
	)
	.fetch_all(&data.db)
	.await;

	if query_result.is_err() {
		let message = "Что-то пошло не так во время чтения пользователей";
		return HttpResponse::InternalServerError()
			.json(json!({"status": "error","message": message}));
	}

	if categories_query_result.is_err() {
		let message = "Что-то пошло не так во время чтения категорий цен";
		return HttpResponse::InternalServerError()
			.json(json!({"status": "error","message": message}));
	}

	let prices_count = Count::count(&data.db, table).await.unwrap_or(0);

	let prices_categories = categories_query_result.unwrap();
	let prices_items = query_result.unwrap();

	let json_response = json!({
		"status":  "success",
		"data": json!({
			"prices_categories": &prices_categories.into_iter().map(|price| filter_price_category_record(&price)).collect::<Vec<FilteredPriceCategory>>(),
			"prices_items": &prices_items.into_iter().map(|price| filter_price_record(&price)).collect::<Vec<FilteredPriceItem>>(),
			"prices_count": &prices_count
		})
	});

	HttpResponse::Ok().json(json_response)
}

#[get("/prices_by_url/{id}")]
async fn get_prices_by_url_handler(
	path: Path<String>,
	opts: web::Query<FilterOptions>,
	data: web::Data<AppState>,
	// _: jwt_auth::JwtMiddleware,
) -> impl Responder {
	let limit = opts.limit.unwrap_or(10);
	let offset = (opts.page.unwrap_or(1) - 1) * limit;
	let table = String::from("prices_items");

	let firm_url = &path.into_inner();
	let firm_query_result = Firm::get_firm_by_url(&data.db, &firm_url).await;
	let firm_message = "Что-то пошло не так во время чтения get_firm_by_url";
	if firm_query_result.is_err() {
		return HttpResponse::InternalServerError()
			.json(json!({"status": "error","message": &firm_message}));
	}
	let firm = firm_query_result.expect(&firm_message);
	let firm_id = firm.firm_id;

	let categories_query_result = sqlx::query_as!(
		PriceCategory,
		"SELECT * FROM prices_categories WHERE firm_id = $1 ORDER by created_ts LIMIT $2 OFFSET $3",
		firm_id,
		limit as i32,
		offset as i32
	)
	.fetch_all(&data.db)
	.await;

	let query_result = sqlx::query_as!(
		PriceItem,
		"SELECT * FROM prices_items WHERE firm_id = $1 ORDER by created_ts LIMIT $2 OFFSET $3",
		firm_id,
		limit as i32,
		offset as i32
	)
	.fetch_all(&data.db)
	.await;

	if query_result.is_err() {
		let message = "Что-то пошло не так во время чтения PriceItem";
		return HttpResponse::InternalServerError()
			.json(json!({"status": "error","message": message}));
	}

	if categories_query_result.is_err() {
		let message = "Что-то пошло не так во время чтения категорий цен";
		return HttpResponse::InternalServerError()
			.json(json!({"status": "error","message": message}));
	}

	// TODO: посчитать нормально кол-во цен
	let prices_count = Count::count(&data.db, table).await.unwrap_or(0);

	let prices_categories = categories_query_result.unwrap();
	let prices_items = query_result.unwrap();

	let json_response = json!({
		"status":  "success",
		"data": json!({
			"prices_categories": &prices_categories.into_iter().map(|price| filter_price_category_record(&price)).collect::<Vec<FilteredPriceCategory>>(),
			"prices_items": &prices_items.into_iter().map(|price| filter_price_record(&price)).collect::<Vec<FilteredPriceItem>>(),
			"prices_count": &prices_count
		})
	});

	HttpResponse::Ok().json(json_response)
}
