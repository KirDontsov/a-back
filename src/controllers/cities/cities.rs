use crate::{
	models::{City, Count, FilterOptions, FilteredCity},
	AppState,
};
use actix_web::{
	get,
	web::{self, Path},
	HttpResponse, Responder,
};
use serde_json::json;
use uuid::Uuid;

use crate::utils::filter_city_record;

#[get("/cities")]
async fn get_cities_handler(
	opts: web::Query<FilterOptions>,
	data: web::Data<AppState>,
) -> impl Responder {
	let limit = opts.limit.unwrap_or(10);
	let offset = (opts.page.unwrap_or(1) - 1) * limit;
	let table = String::from("cities");

	let query_result = sqlx::query_as!(
		City,
		"SELECT * FROM cities ORDER by order_number LIMIT $1 OFFSET $2",
		limit as i32,
		offset as i32
	)
	.fetch_all(&data.db)
	.await;

	let city_count = Count::count(&data.db, table).await.unwrap_or(0);

	if query_result.is_err() {
		let message = "Что-то пошло не так во время чтения cities";
		return HttpResponse::InternalServerError()
			.json(json!({"status": "error","message": message}));
	}

	let cities = query_result.unwrap();

	let json_response = json!({
		"status":  "success",
		"data": json!({
			"cities": &cities.into_iter().map(|city| filter_city_record(&city)).collect::<Vec<FilteredCity>>(),
			"cities_count": &city_count
		})
	});

	HttpResponse::Ok().json(json_response)
}

#[get("/city/{id}")]
async fn get_city_handler(path: Path<Uuid>, data: web::Data<AppState>) -> impl Responder {
	let city_id = &path.into_inner();

	let city = sqlx::query_as!(City, "SELECT * FROM cities WHERE city_id = $1", city_id)
		.fetch_one(&data.db)
		.await
		.unwrap();

	let json_response = json!({
		"status":  "success",
		"data": json!({
			"city": filter_city_record(&city)
		})
	});

	HttpResponse::Ok().json(json_response)
}
