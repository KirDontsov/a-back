use crate::{
	models::{Category, Count, FilterOptions, FilteredCategory},
	AppState,
};
use actix_web::{
	get,
	web::{self, Path},
	HttpResponse, Responder,
};
use serde_json::json;
use uuid::Uuid;

use crate::utils::filter_category_record;

#[get("/categories")]
async fn get_categories_handler(
	opts: web::Query<FilterOptions>,
	data: web::Data<AppState>,
) -> impl Responder {
	let limit = opts.limit.unwrap_or(10);
	let offset = (opts.page.unwrap_or(1) - 1) * limit;
	let table = String::from("categories");

	let query_result = sqlx::query_as!(
		Category,
		"SELECT * FROM categories ORDER by order_number LIMIT $1 OFFSET $2",
		limit as i32,
		offset as i32
	)
	.fetch_all(&data.db)
	.await;

	let category_count = Count::count(&data.db, table).await.unwrap_or(0);

	let message = "Что-то пошло не так во время чтения categories";
	if query_result.is_err() {
		return HttpResponse::InternalServerError()
			.json(json!({"status": "error","message": &message}));
	}

	let categories = query_result.expect(&message);

	let json_response = json!({
		"status":  "success",
		"data": json!({
			"categories": &categories.into_iter().map(|category| filter_category_record(&category)).collect::<Vec<FilteredCategory>>(),
			"categories_count": &category_count
		})
	});

	HttpResponse::Ok().json(json_response)
}

#[get("/category/{id}")]
async fn get_category_handler(path: Path<Uuid>, data: web::Data<AppState>) -> impl Responder {
	let category_id = &path.into_inner();

	let category = sqlx::query_as!(
		Category,
		"SELECT * FROM categories WHERE category_id = $1",
		category_id
	)
	.fetch_one(&data.db)
	.await
	.unwrap();

	let json_response = json!({
		"status":  "success",
		"data": json!({
			"category": filter_category_record(&category)
		})
	});

	HttpResponse::Ok().json(json_response)
}

#[get("/category_abbr/{id}")]
async fn get_category_by_abbreviation_handler(
	path: Path<String>,
	data: web::Data<AppState>,
) -> impl Responder {
	let category_abbreviation = &path.into_inner();

	let category_query_result =
		Category::get_category_by_abbr(&data.db, &category_abbreviation).await;
	let category_message = "Что-то пошло не так во время чтения category";
	if category_query_result.is_err() {
		return HttpResponse::InternalServerError()
			.json(json!({"status": "error","message": &category_message}));
	}

	let category = category_query_result.expect(&category_message);

	let json_response = json!({
		"status":  "success",
		"data": json!({
			"category": filter_category_record(&category)
		})
	});

	HttpResponse::Ok().json(json_response)
}
