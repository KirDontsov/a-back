use crate::{
	jwt_auth,
	models::{Count, FilterOptions, FilteredUser, UpdateUserSchema, User},
	AppState,
};
use actix_web::{
	get, put,
	web::{self, Path},
	HttpResponse, Responder,
};
use actix_web_grants::proc_macro::has_any_role;
use serde_json::json;
use uuid::Uuid;

use crate::controllers::auth::Role;
use crate::utils::filter_user_record;

#[get("/users")]
#[has_any_role("Role::Admin", type = "Role")]
async fn get_users_handler(
	opts: web::Query<FilterOptions>,
	data: web::Data<AppState>,
	_: jwt_auth::JwtMiddleware,
) -> impl Responder {
	let limit = opts.limit.unwrap_or(10);
	let offset = (opts.page.unwrap_or(1) - 1) * limit;
	let table = String::from("users");

	let query_result = sqlx::query_as!(
		User,
		"SELECT * FROM users ORDER by id LIMIT $1 OFFSET $2",
		limit as i32,
		offset as i32
	)
	.fetch_all(&data.db)
	.await;

	let users_count = Count::count(&data.db, table).await.unwrap_or(0);

	if query_result.is_err() {
		let message = "Что-то пошло не так во время чтения пользователей";
		return HttpResponse::InternalServerError()
			.json(json!({"status": "error","message": message}));
	}

	let users = query_result.unwrap();

	let json_response = json!({
		"status":  "success",
		"data": json!({
			"users": &users.into_iter().map(|user| filter_user_record(&user)).collect::<Vec<FilteredUser>>(),
			"users_count": &users_count
		})
	});

	HttpResponse::Ok().json(json_response)
}

#[get("/user/{id}")]
#[has_any_role("Role::Admin", type = "Role")]
async fn get_user_handler(
	path: Path<Uuid>,
	data: web::Data<AppState>,
	_: jwt_auth::JwtMiddleware,
) -> impl Responder {
	let user_id = &path.into_inner();

	let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
		.fetch_one(&data.db)
		.await
		.unwrap();

	let json_response = json!({
		"status":  "success",
		"data": json!({
			"user": filter_user_record(&user)
		})
	});

	HttpResponse::Ok().json(json_response)
}

#[put("/user/{id}")]
#[has_any_role("Role::Admin", type = "Role")]
async fn update_user_handler(
	path: Path<Uuid>,
	opts: web::Json<UpdateUserSchema>,
	data: web::Data<AppState>,
	_: jwt_auth::JwtMiddleware,
) -> impl Responder {
	let id = &path.into_inner();
	let name = &opts.name;
	let email = &opts.email;
	let role = &opts.role;
	let verified = &opts.verified;
	let favourite = &opts.favourite;
	// let updated_at = Utc::now();

	// Create longer-lived values to avoid temporary value issues
	let name_value = name.as_deref().unwrap_or_default();
	let email_value = email.as_deref().unwrap_or_default();
	let role_value = role.as_deref().unwrap_or_default();
	let verified_value = verified.unwrap_or_default();
	let empty_vec = Vec::new();
	let favourite_value = favourite.as_ref().unwrap_or(&empty_vec);

	let user = sqlx::query_as!(
		User,
		r#"UPDATE users SET name = $1, email = $2, role = $3, verified = $4, favourite = $5 WHERE id = $6 RETURNING *"#,
		name_value,
	email_value,
		role_value,
	verified_value,
	favourite_value,
		// updated_at,
		id
	)
	.fetch_one(&data.db)
	.await
	.unwrap();

	let json_response = json!({
		"status":  "success",
		"data": json!({
			"user": filter_user_record(&user)
		})
	});

	HttpResponse::Ok().json(json_response)
}
