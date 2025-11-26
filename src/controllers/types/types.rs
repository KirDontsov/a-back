use crate::{
	models::{Count, FilterOptions, FilteredType, Type},
	AppState,
};
use actix_web::{
	get,
	web::{self, Path},
	HttpResponse, Responder,
};
use serde_json::json;
use uuid::Uuid;

use crate::utils::filter_type_record;

#[get("/types")]
async fn get_types_handler(
	opts: web::Query<FilterOptions>,
	data: web::Data<AppState>,
) -> impl Responder {
	let limit = opts.limit.unwrap_or(10);
	let offset = (opts.page.unwrap_or(1) - 1) * limit;
	let table = String::from("types");

	let query_result = sqlx::query_as!(
		Type,
		"SELECT * FROM types ORDER by type_id LIMIT $1 OFFSET $2",
		limit as i32,
		offset as i32
	)
	.fetch_all(&data.db)
	.await;

	let types_count = Count::count(&data.db, table).await.unwrap_or(0);

	if query_result.is_err() {
		let message = "Что-то пошло не так во время чтения types";
		return HttpResponse::InternalServerError()
			.json(json!({"status": "error","message": message}));
	}

	let types = query_result.unwrap();

	let json_response = json!({
		"status":  "success",
		"data": json!({
			"types": &types.into_iter().map(|type_item| filter_type_record(&type_item)).collect::<Vec<FilteredType>>(),
			"types_count": &types_count
		})
	});

	HttpResponse::Ok().json(json_response)
}

#[get("/type/{id}")]
async fn get_type_handler(path: Path<Uuid>, data: web::Data<AppState>) -> impl Responder {
	let type_id = &path.into_inner();

	let type_item = sqlx::query_as!(Type, "SELECT * FROM types WHERE type_id = $1", type_id)
		.fetch_one(&data.db)
		.await
		.unwrap();

	let json_response = json!({
		"status":  "success",
		"data": json!({
			"type": filter_type_record(&type_item)
		})
	});

	HttpResponse::Ok().json(json_response)
}
