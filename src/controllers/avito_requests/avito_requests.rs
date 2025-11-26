use crate::controllers::auth::Role;
use crate::utils::avito_requests::filter_add_avito_request_record;
use crate::{
	jwt_auth::JwtMiddleware,
	models::{
		AdRecord, AvitoRequest, Count, FilterOptions, FilteredAvitoRequest, SaveAvitoRequest,
	},
	AppState,
};
use actix_web::{
	get, post,
	web::{self, Path},
	HttpResponse, Responder,
};
use actix_web_grants::proc_macro::has_any_role;
use serde_json::json;
use uuid::Uuid;

use crate::utils::transliterate::Translit;
use csv::Writer;
use serde::{Deserialize, Serialize};
use std::io::Cursor;

#[derive(Debug, Serialize, Deserialize)]
pub struct AvitoRequestMessage {
	pub request_id: Uuid,
	pub user_id: Uuid,
	pub request: String,
	pub city: String,
	pub coords: String,
	pub radius: String,
	pub district: String,
	pub created_ts: chrono::DateTime<chrono::Utc>,
}

// Get my account avito requests
#[get("/avito_requests/{id}")]
#[has_any_role("Role::Admin", type = "Role")]
async fn get_avito_requests_handler(
	path: Path<Uuid>,
	opts: web::Query<FilterOptions>,
	data: web::Data<AppState>,
	// _: jwt_auth::JwtMiddleware,
) -> impl Responder {
	let user_id = &path.into_inner();
	let limit = opts.limit.unwrap_or(10);
	let offset = (opts.page.unwrap_or(1) - 1) * limit;
	let table = String::from("avito_requests");

	let query_result =
		AvitoRequest::get_avito_requests_by_user(&data.db, user_id, limit as i64, offset as i64)
			.await;
	let reviews_message = "Что-то пошло не так во время чтения category";
	if query_result.is_err() {
		return HttpResponse::InternalServerError()
			.json(json!({"status": "error","message": &reviews_message}));
	}
	let reviews = query_result.expect(&reviews_message);

	let avito_requests_count = Count::count(&data.db, table).await.unwrap_or(0);

	let json_response = json!({
		"status":  "success",
		"data": json!({
			"avito_requests": &reviews.into_iter().map(|review| filter_add_avito_request_record(&review)).collect::<Vec<FilteredAvitoRequest>>(),
			"avito_requests_count": &avito_requests_count,
		})
	});

	HttpResponse::Ok().json(json_response)
}

// Create avito request
#[post("/avito_requests/{id}")]
#[has_any_role("Role::Admin", type = "Role")]
async fn create_avito_request_handler(
	path: Path<Uuid>,
	body: web::Json<SaveAvitoRequest>,
	data: web::Data<AppState>,
	_: JwtMiddleware,
) -> impl Responder {
	let user_id = path.into_inner();
	let request = &body.request;
	let city = &body.city;
	let coords = &body.coords;
	let radius = &body.radius;
	let district = &body.district;

	let query_result = sqlx::query_as!(
        AvitoRequest,
        "INSERT INTO avito_requests (user_id, request, city, coords, radius, district) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
        user_id,
        request.to_string(),
        city.to_string(),
        coords.to_string(),
        radius.to_string(),
        district.to_string(),
    )
		.fetch_one(&data.db)
		.await;

	match query_result {
		Ok(avito_request) => {
			// Create message
			let message = AvitoRequestMessage {
				request_id: avito_request.request_id.clone(),
				user_id: avito_request.user_id.clone(),
				request: avito_request.request.clone().expect("REASON"),
				city: avito_request.city.clone().expect("REASON"),
				coords: avito_request.coords.clone().expect("REASON"),
				radius: avito_request.radius.clone().expect("REASON"),
				district: avito_request.district.clone().expect("REASON"),
				created_ts: avito_request.created_ts.clone().expect("REASON"),
			};

			// Publish to RabbitMQ
			match publish_avito_request(&data.rabbitmq_channel, &message).await {
				Ok(_) => {
					let avito_request_response = serde_json::json!({
						"status": "success",
						"data": serde_json::json!({
							"avito_request": filter_add_avito_request_record(&avito_request.clone())
						})
					});
					HttpResponse::Ok().json(avito_request_response)
				}
				Err(e) => {
					log::error!("Failed to publish message: {}", e);
					// You might want to handle this differently - maybe still return success
					// but log the error, or return a partial success response
					HttpResponse::Accepted().json(serde_json::json!({
						"status": "success",
						"message": "Request created but notification failed"
					}))
				}
			}
		}
		Err(e) => HttpResponse::InternalServerError()
			.json(serde_json::json!({"status": "error","message": format!("{:?}", e)})),
	}
}

// Message publishing function
async fn publish_avito_request(
	channel: &lapin::Channel,
	message: &AvitoRequestMessage,
) -> Result<(), Box<dyn std::error::Error>> {
	let message_json = serde_json::to_string(message)?;

	// Declare exchange
	channel
		.exchange_declare(
			"avito_exchange",
			lapin::ExchangeKind::Topic,
			lapin::options::ExchangeDeclareOptions {
				durable: true,
				..lapin::options::ExchangeDeclareOptions::default()
			},
			lapin::types::FieldTable::default(),
		)
		.await?;

	// Publish to exchange with routing key including user_id
	channel
		.basic_publish(
			"avito_exchange",                           // exchange name
			&format!("task.crawl.{}", message.user_id), // routing key for crawl tasks
			lapin::options::BasicPublishOptions::default(),
			message_json.as_bytes(),
			lapin::BasicProperties::default(),
		)
		.await?;

	log::info!(
		"Published Avito request message for user: {} with routing key: task.crawl.{}",
		message.user_id,
		message.user_id
	);
	Ok(())
}

// GET avito request with ads
#[get("/avito_requests/{avito_request_id}/ads")]
#[has_any_role("Role::Admin", type = "Role")]
async fn get_ads_by_avito_request_id_handler(
	path: Path<Uuid>,
	opts: web::Query<FilterOptions>,
	data: web::Data<AppState>,
	_: JwtMiddleware,
) -> impl Responder {
	let avito_request_id = path.into_inner();
	let limit = opts.limit.unwrap_or(10);
	let offset = (opts.page.unwrap_or(1) - 1) * limit;

	let query = "
	           SELECT ad_id, my_ad, run_date, city_query, search_query, position, views, views_today,
	                  promotion, delivery, ad_date, avito_ad_id, title, price, link, categories,
	                  seller_id, seller_name, seller_type, register_date, answer_time,
	                  rating, reviews_count, ads_count, closed_ads_count, photo_count,
	                  address, description, avito_request_id, created_ts
	           FROM avito_analytics_ads
	           WHERE avito_request_id = $1
	           ORDER BY position
	           LIMIT $2 OFFSET $3
	       ";

	let query_result = sqlx::query_as::<_, AdRecord>(query)
		.bind(avito_request_id)
		.bind(limit as i64)
		.bind(offset as i64)
		.fetch_all(&data.db)
		.await;

	match query_result {
		Ok(ads) => {
			// Get total count of ads for this request
			let ads_count = sqlx::query_scalar!(
				"SELECT COUNT(*) FROM avito_analytics_ads WHERE avito_request_id = $1",
				avito_request_id
			)
			.fetch_one(&data.db)
			.await
			.unwrap_or(Some(0i64))
			.unwrap_or(0i64);

			let json_response = serde_json::json!({
				"status": "success",
				"data": json!({
					"ads": &ads,
					"ads_count": &ads_count
				})
			});
			HttpResponse::Ok().json(json_response)
		}
		Err(e) => HttpResponse::InternalServerError()
			.json(serde_json::json!({"status": "error","message": format!("{:?}", e)})),
	}
}

// GET avito request with ads in a csv file
#[get("/avito_requests/{avito_request_id}/ads/csv")]
#[has_any_role("Role::Admin", type = "Role")]
async fn get_ads_by_avito_request_id_csv_handler(
	path: Path<Uuid>,
	data: web::Data<AppState>,
	_: JwtMiddleware,
) -> impl Responder {
	let avito_request_id = path.into_inner();

	let query = "
	           SELECT ad_id, my_ad, run_date, city_query, search_query, position, views, views_today,
	                  promotion, delivery, ad_date, avito_ad_id, title, price, link, categories,
	                  seller_id, seller_name, seller_type, register_date, answer_time,
	                  rating, reviews_count, ads_count, closed_ads_count, photo_count,
	                  address, description, avito_request_id, created_ts
	           FROM avito_analytics_ads
	           WHERE avito_request_id = $1
	           ORDER BY position
	       ";

	let query_result = sqlx::query_as::<_, AdRecord>(query)
		.bind(avito_request_id)
		.fetch_all(&data.db)
		.await;

	match query_result {
		Ok(ads) => {
			// Create CSV in memory
			let mut writer = Writer::from_writer(Cursor::new(Vec::new()));

			// Write headers
			writer
				.write_record(&[
					"Мое",
					"Дата прогона",
					"Город (запрос)",
					"Поиск (запрос)",
					"Поз.",
					"Просмотров",
					"Просмотров сегодня",
					"Продвижение",
					"Доставка",
					"Дата объявления",
					"id",
					"Название",
					"Цена",
					"Ссылка",
					"Категории",
					"id Продавца",
					"Продавец",
					"Тип продавца",
					"Дата регистрации",
					"Время ответа",
					"Рейтинг",
					"Кол. отзывов",
					"Кол. объявлений",
					"Кол. закрытых",
					"Фото",
					"Адрес",
					"Описание",
				])
				.unwrap();

			// Write records
			for ad in &ads {
				writer
					.write_record(&[
						ad.my_ad.as_str(),
						ad.run_date.to_rfc3339().as_str(),
						ad.city_query.as_str(),
						ad.search_query.as_str(),
						ad.position.to_string().as_str(),
						ad.views.as_str(),
						ad.views_today.as_str(),
						ad.promotion.as_str(),
						ad.delivery.as_str(),
						ad.ad_date.as_str(),
						ad.avito_ad_id.as_str(),
						ad.title.as_str(),
						ad.price.as_str(),
						ad.link.as_str(),
						ad.categories.as_str(),
						ad.seller_id.as_str(),
						ad.seller_name.as_str(),
						ad.seller_type.as_str(),
						ad.register_date.as_str(),
						ad.answer_time.as_str(),
						ad.rating.as_str(),
						ad.reviews_count.as_str(),
						ad.ads_count.as_str(),
						ad.closed_ads_count.as_str(),
						ad.photo_count.as_str(),
						ad.address.as_str(),
						ad.description.as_str(),
					])
					.unwrap();
			}

			// Get the CSV bytes
			let csv_bytes = writer.into_inner().unwrap().into_inner();

			// Generate filename using search_query and date
			let filename = if !ads.is_empty() {
				let first_ad = &ads[0];
				let search_query = first_ad.search_query.clone();
				// Replace invalid filename characters with underscores
				let sanitized_query = search_query
					.chars()
					.map(|c| match c {
						'/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0' => '_',
						_ => c,
					})
					.collect::<String>();
				let transliterated_query = Translit::convert(Some(sanitized_query));
				let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
				format!("{}_{}.csv", transliterated_query, date)
			} else {
				let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
				format!("ads_{}.csv", date)
			};

			// Create response with CSV content type
			HttpResponse::Ok()
				.content_type("text/csv")
				.append_header((
					"Content-Disposition",
					format!("attachment; filename=\"{}\"", filename),
				))
				.body(csv_bytes)
		}
		Err(e) => HttpResponse::InternalServerError()
			.json(serde_json::json!({"status": "error","message": format!("{:?}", e)})),
	}
}

// GET all accaunts avito requests
#[get("/avito_requests")]
#[has_any_role("Role::Admin", type = "Role")]
async fn get_all_avito_requests_handler(
	opts: web::Query<FilterOptions>,
	data: web::Data<AppState>,
	_: JwtMiddleware,
) -> impl Responder {
	let limit = opts.limit.unwrap_or(10);
	let offset = (opts.page.unwrap_or(1) - 1) * limit;
	let table = String::from("avito_requests");

	let query_result = sqlx::query_as!(
		AvitoRequest,
		"SELECT * FROM avito_requests ORDER BY created_ts DESC LIMIT $1 OFFSET $2",
		limit as i64,
		offset as i64
	)
	.fetch_all(&data.db)
	.await;

	let error_message = "Error fetching avito requests";
	if query_result.is_err() {
		return HttpResponse::InternalServerError()
			.json(json!({"status": "error", "message": error_message}));
	}

	let avito_requests = query_result.expect(error_message);
	let avito_requests_count = Count::count(&data.db, table).await.unwrap_or(0);

	let json_response = json!({
		"status": "success",
		"data": json!({
			"avito_requests": &avito_requests.into_iter().map(|request| filter_add_avito_request_record(&request)).collect::<Vec<FilteredAvitoRequest>>(),
			"avito_requests_count": &avito_requests_count,
		})
	});

	HttpResponse::Ok().json(json_response)
}

// GET all avito requests by specific user_id
#[get("/avito_requests/user/{user_id}")]
async fn get_avito_requests_by_user_handler(
	opts: web::Query<FilterOptions>,
	path: Path<Uuid>,
	data: web::Data<AppState>,
	user: JwtMiddleware,
) -> impl Responder {
	let requested_user_id = path.into_inner();
	let current_user_id = user.user_id; // Get the authenticated user's ID
	let limit = opts.limit.unwrap_or(20);
	let offset = (opts.page.unwrap_or(1) - 1) * limit;

	// Check if the requested user_id matches the authenticated user's ID
	if requested_user_id != current_user_id {
		return HttpResponse::Forbidden()
			.json(json!({"status": "error", "message": "Access denied. You can only access your own avito requests."}));
	}

	let query_result = sqlx::query_as!(
	AvitoRequest,
		"SELECT * FROM avito_requests WHERE user_id = $1 ORDER BY created_ts DESC LIMIT $2 OFFSET $3",
		requested_user_id,
		limit as i64,
		offset as i64
	)
	.fetch_all(&data.db)
	.await;

	let error_message = "Error fetching avito requests for user";
	if query_result.is_err() {
		return HttpResponse::InternalServerError()
			.json(json!({"status": "error", "message": error_message}));
	}

	let avito_requests = query_result.expect(error_message);
	let avito_requests_count = sqlx::query_scalar!(
		"SELECT COUNT(*) FROM avito_requests WHERE user_id = $1",
		requested_user_id
	)
	.fetch_one(&data.db)
	.await
	.unwrap_or(Some(0i64))
	.unwrap_or(0i64);

	let json_response = json!({
		"status": "success",
	"data": json!({
			"avito_requests": &avito_requests.into_iter().map(|request| filter_add_avito_request_record(&request)).collect::<Vec<FilteredAvitoRequest>>(),
			"avito_requests_count": &avito_requests_count,
		})
	});

	HttpResponse::Ok().json(json_response)
}
