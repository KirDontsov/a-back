use crate::{
	models::{AddReview, Count, FilterOptions, FilteredReview, Firm, Review},
	AppState,
};
use actix_web::{
	get, post,
	web::{self, Path},
	HttpResponse, Responder,
};
use serde_json::json;
use uuid::Uuid;

use crate::utils::{filter_add_review_record, filter_review_record};

#[get("/reviews/{id}")]
async fn get_reviews_handler(
	path: Path<Uuid>,
	opts: web::Query<FilterOptions>,
	data: web::Data<AppState>,
	// _: jwt_auth::JwtMiddleware,
) -> impl Responder {
	let firm_id = &path.into_inner();
	let limit = opts.limit.unwrap_or(10);
	let offset = (opts.page.unwrap_or(1) - 1) * limit;
	let table = String::from("reviews");

	let query_result =
		Review::get_reviews_by_firm(&data.db, firm_id, limit as i64, offset as i64).await;
	let reviews_message = "Что-то пошло не так во время чтения category";
	if query_result.is_err() {
		return HttpResponse::InternalServerError()
			.json(json!({"status": "error","message": &reviews_message}));
	}
	let reviews = query_result.expect(&reviews_message);

	let reviews_count = Count::count_smth_by_firm(&data.db, table, firm_id)
		.await
		.unwrap_or(0);

	let json_response = json!({
		"status":  "success",
		"data": json!({
			"reviews": &reviews.into_iter().map(|review| filter_review_record(&review)).collect::<Vec<FilteredReview>>(),
			"reviews_count": &reviews_count,
		})
	});

	HttpResponse::Ok().json(json_response)
}

#[get("/reviews_by_url/{id}")]
async fn get_reviews_by_url_handler(
	path: Path<String>,
	opts: web::Query<FilterOptions>,
	data: web::Data<AppState>,
	// _: jwt_auth::JwtMiddleware,
) -> impl Responder {
	let limit = opts.limit.unwrap_or(10);
	let offset = (opts.page.unwrap_or(1) - 1) * limit;
	let table = String::from("reviews");

	let firm_url = &path.into_inner();
	let firm_query_result = Firm::get_firm_by_url(&data.db, &firm_url).await;
	let firm_message = "Что-то пошло не так во время чтения get_firm_by_url";
	if firm_query_result.is_err() {
		return HttpResponse::InternalServerError()
			.json(json!({"status": "error","message": &firm_message}));
	}
	let firm = firm_query_result.expect(&firm_message);
	let firm_id = firm.firm_id;

	let query_result =
		Review::get_reviews_by_firm(&data.db, &firm_id, limit as i64, offset as i64).await;
	let reviews_message = "Что-то пошло не так во время чтения get_reviews_by_firm";
	if query_result.is_err() {
		return HttpResponse::InternalServerError()
			.json(json!({"status": "error","message": &reviews_message}));
	}
	let reviews = query_result.expect(&reviews_message);

	let reviews_count = Count::count_smth_by_firm(&data.db, table, &firm_id)
		.await
		.unwrap_or(0);

	let json_response = json!({
		"status":  "success",
		"data": json!({
			"reviews": &reviews.into_iter().map(|review| filter_review_record(&review)).collect::<Vec<FilteredReview>>(),
			"reviews_count": &reviews_count,
		})
	});

	HttpResponse::Ok().json(json_response)
}

#[get("/review/{id}")]
async fn get_review_handler(
	path: Path<Uuid>,
	data: web::Data<AppState>,
	// _: jwt_auth::JwtMiddleware,
) -> impl Responder {
	let review_id = &path.into_inner();

	let review = sqlx::query_as!(
		Review,
		"SELECT * FROM reviews WHERE review_id = $1",
		review_id
	)
	.fetch_one(&data.db)
	.await
	.unwrap();

	let json_response = json!({
		"status":  "success",
		"data": json!({
			"review": filter_review_record(&review)
		})
	});

	HttpResponse::Ok().json(json_response)
}

#[post("/review/{id}")]
async fn add_review_handler(
	body: web::Json<AddReview>,
	data: web::Data<AppState>,
) -> impl Responder {
	let firm_id = uuid::Uuid::parse_str(body.firm_id.to_string().as_str()).unwrap();

	let query_result = sqlx::query_as!(
		Review,
		"INSERT INTO reviews (firm_id, text, author, rating, parsed) VALUES ($1, $2, $3, $4, $5) RETURNING *",
		firm_id,
		body.text.to_string(),
		body.author.to_string(),
		body.rating.to_string().to_lowercase(),
		false
	)
	.fetch_one(&data.db)
	.await;

	match query_result {
		Ok(review) => {
			let review_response = serde_json::json!({"status": "success","data": serde_json::json!({
				"review": filter_add_review_record(&review)
			})});

			return HttpResponse::Ok().json(review_response);
		}
		Err(e) => {
			return HttpResponse::InternalServerError()
				.json(serde_json::json!({"status": "error","message": format!("{:?}", e)}));
		}
	}
}
