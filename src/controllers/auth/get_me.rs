use crate::{jwt_auth, models::User, AppState};
use actix_web::{get, web, HttpMessage, HttpRequest, HttpResponse, Responder};

use crate::utils::filter_user_record;

#[get("/users/me")]
async fn get_me_handler(
	req: HttpRequest,
	data: web::Data<AppState>,
	_: jwt_auth::JwtMiddleware,
) -> impl Responder {
	let ext = req.extensions();
	let user_id = ext.get::<uuid::Uuid>().unwrap();

	let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
		.fetch_one(&data.db)
		.await
		.unwrap();

	let json_response = serde_json::json!({
		"status":  "success",
		"data": serde_json::json!({
			"user": filter_user_record(&user)
		})
	});

	HttpResponse::Ok().json(json_response)
}
