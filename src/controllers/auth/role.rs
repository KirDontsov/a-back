use actix_web::dev::ServiceRequest;
use actix_web::{http, web, Error, HttpMessage};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::models::{TokenClaims, User};
use crate::AppState;

#[derive(PartialEq, Clone, Debug, Deserialize, Serialize)]
pub enum Role {
	Admin,
	Manager,
}

pub async fn extract(req: &mut ServiceRequest) -> Result<Vec<Role>, Error> {
	let data = req.app_data::<web::Data<AppState>>().unwrap();

	let token = req
		.cookie("token")
		.map(|c| c.value().to_string())
		.or_else(|| {
			req.headers()
				.get(http::header::AUTHORIZATION)
				.map(|h| h.to_str().unwrap().split_at(7).1.to_string())
		});

	if token.is_none() {
		return Ok(vec![Role::Manager]);
	}

	let claims = decode::<TokenClaims>(
		&token.unwrap(),
		&DecodingKey::from_secret(data.env.jwt_secret.as_ref()),
		&Validation::default(),
	)
	.unwrap();

	let user_id = uuid::Uuid::parse_str(claims.claims.sub.as_str()).unwrap();

	let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
		.fetch_one(&data.db)
		.await
		.unwrap();

	if user.role.as_deref() != Some("admin") {
		Ok(vec![Role::Manager])
	} else {
		Ok(vec![Role::Admin])
	}
}
