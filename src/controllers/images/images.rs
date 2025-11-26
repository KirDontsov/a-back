use crate::{
	models::{Count, FilteredImage, Firm, Image},
	utils::filter_image_record::filter_image_record,
	AppState,
};
use actix_web::{
	get,
	web::{self, Path},
	HttpResponse, Responder,
};
use serde_json::json;
use uuid::Uuid;

#[get("/images/{id}")]
async fn get_images_handler(path: Path<Uuid>, data: web::Data<AppState>) -> impl Responder {
	let firm_id = &path.into_inner();
	let table = String::from("images");

	let query_result = sqlx::query_as!(Image, "SELECT * FROM images WHERE firm_id = $1", firm_id)
		.fetch_all(&data.db)
		.await;

	let images_count = Count::count(&data.db, table).await.unwrap_or(0);

	if query_result.is_err() {
		let message = "Что-то пошло не так во время чтения пользователей";
		return HttpResponse::InternalServerError()
			.json(json!({"status": "error","message": message}));
	}

	let images = query_result.unwrap();

	let json_response = json!({
		"status":  "success",
		"data": json!({
			"images": &images.into_iter().map(|image| filter_image_record(&image)).collect::<Vec<FilteredImage>>(),
			"images_count": &images_count
		})
	});

	HttpResponse::Ok().json(json_response)
}

#[get("/images_by_url/{id}")]
async fn get_images_by_url_handler(
	path: Path<String>,
	data: web::Data<AppState>,
) -> impl Responder {
	let table = String::from("images");
	let firm_url = &path.into_inner();
	let firm_query_result = Firm::get_firm_by_url(&data.db, &firm_url).await;
	let firm_message = "Что-то пошло не так во время чтения get_firm_by_url";
	if firm_query_result.is_err() {
		return HttpResponse::InternalServerError()
			.json(json!({"status": "error","message": &firm_message}));
	}
	let firm = firm_query_result.expect(&firm_message);
	let firm_id = firm.firm_id;

	let query_result = sqlx::query_as!(Image, "SELECT * FROM images WHERE firm_id = $1", firm_id)
		.fetch_all(&data.db)
		.await;

	let images_count = Count::count(&data.db, table).await.unwrap_or(0);

	if query_result.is_err() {
		let message = "Что-то пошло не так во время чтения images";
		return HttpResponse::InternalServerError()
			.json(json!({"status": "error","message": message}));
	}

	let images = query_result.unwrap();

	let json_response = json!({
		"status":  "success",
		"data": json!({
			"images": &images.into_iter().map(|image| filter_image_record(&image)).collect::<Vec<FilteredImage>>(),
			"images_count": &images_count
		})
	});

	HttpResponse::Ok().json(json_response)
}

#[get("/image_by_url/{id}")]
async fn get_image_by_url_handler(path: Path<String>, data: web::Data<AppState>) -> impl Responder {
	let table = String::from("images");
	let firm_url = &path.into_inner();
	let firm_query_result = Firm::get_firm_by_url(&data.db, &firm_url).await;
	let firm_message = "Что-то пошло не так во время чтения get_firm_by_url";
	if firm_query_result.is_err() {
		return HttpResponse::InternalServerError()
			.json(json!({"status": "error","message": &firm_message}));
	}
	let firm = firm_query_result.expect(&firm_message);
	let firm_id = firm.firm_id;

	let query_result = sqlx::query_as!(
		Image,
		"SELECT * FROM images WHERE firm_id = $1 LIMIT 1 OFFSET 0",
		firm_id
	)
	.fetch_all(&data.db)
	.await;

	let images_count = Count::count(&data.db, table).await.unwrap_or(0);

	if query_result.is_err() {
		let message = "Что-то пошло не так во время чтения image";
		return HttpResponse::InternalServerError()
			.json(json!({"status": "error","message": message}));
	}

	let images = query_result.unwrap();

	let json_response = json!({
		"status":  "success",
		"data": json!({
			"images": &images.into_iter().map(|image| filter_image_record(&image)).collect::<Vec<FilteredImage>>(),
			"images_count": &images_count
		})
	});

	HttpResponse::Ok().json(json_response)
}

#[get("/image/{id}")]
async fn get_image_handler(path: Path<Uuid>, data: web::Data<AppState>) -> impl Responder {
	let img_id = &path.into_inner();

	let image = sqlx::query_as!(Image, "SELECT * FROM images WHERE img_id = $1", img_id)
		.fetch_one(&data.db)
		.await
		.unwrap();

	let json_response = json!({
		"status":  "success",
		"data": json!({
			"image": filter_image_record(&image)
		})
	});

	HttpResponse::Ok().json(json_response)
}
