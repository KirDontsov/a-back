use crate::jwt_auth;
use actix_web::{
	get, HttpRequest, HttpResponse, Responder,
};
use serde_json::json;

#[get("/auth/logout")]
async fn logout_handler(
	req: HttpRequest,
	_: jwt_auth::JwtMiddleware,
) -> impl Responder {

	HttpResponse::Ok()
		.json(json!({"status": "success"}))
}
