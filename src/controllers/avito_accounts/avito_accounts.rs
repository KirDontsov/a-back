use crate::utils::avito_requests::{get_avito_token, get_client_id_from_avito};
use crate::utils::encryption::{decrypt_data, encrypt_data, generate_iv};
use crate::{
	models::{AvitoAccount, CreateAvitoAccountSchema, DbAvitoAccount, UpdateAvitoAccountSchema},
	AppState,
};
use actix_web::{
	delete, get, post, put,
	web::{self, Path},
	HttpResponse, Responder,
};
use serde_json::json;
use uuid::Uuid;

// Global key for encryption (in production, this should be stored securely)
static ENCRYPTION_KEY: [u8; 32] = [
	1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
	27, 28, 29, 30, 31, 32,
];

// Function to combine IV and encrypted data
fn combine_iv_and_data(iv: &[u8; 16], encrypted_data: &str) -> String {
	format!("{}:{}", hex::encode(iv), encrypted_data)
}

// Function to split IV and encrypted data
fn split_iv_and_data(
	combined_data: &str,
) -> Result<([u8; 16], String), Box<dyn std::error::Error>> {
	let parts: Vec<&str> = combined_data.split(':').collect();
	if parts.len() != 2 {
		return Err("Invalid combined data format".into());
	}

	let iv = hex::decode(parts[0])?;
	if iv.len() != 16 {
		return Err("Invalid IV length".into());
	}

	let mut iv_bytes = [0u8; 16];
	iv_bytes.copy_from_slice(&iv);

	Ok((iv_bytes, parts[1].to_string()))
}

#[get("/avito/accounts")]
pub async fn get_avito_accounts_handler(
	data: web::Data<AppState>,
	user: crate::jwt_auth::JwtMiddleware,
) -> impl Responder {
	let query_result = sqlx::query_as!(
        DbAvitoAccount,
        "SELECT account_id, user_id, client_id, avito_client_secret, avito_client_id, is_connected, created_ts, updated_ts FROM avito_accounts WHERE user_id = $1",
        user.user_id.to_string()
    )
    .fetch_all(&data.db)
    .await;

	if query_result.is_err() {
		let message = "Что-то пошло не так во время чтения аккаунтов Avito";
		return HttpResponse::InternalServerError()
			.json(json!({"status": "error","message": message}));
	}

	let db_accounts = query_result.unwrap();
	let accounts: Vec<AvitoAccount> = db_accounts
		.into_iter()
		.map(|db_acc| db_acc.into())
		.collect();

	let json_response = json!({
		"status": "success",
		"data": json!({
			"avito_accounts": &accounts.iter().map(|acc| {
				// Decrypt the credentials for the response
				let decrypted_credentials = decrypt_avito_credentials(&acc.avito_client_secret, &acc.avito_client_id).unwrap_or_else(|_| (String::new(), String::new()));

				json!({
					"account_id": acc.account_id,
					"user_id": &acc.user_id,
					"client_id": &acc.client_id,
					"avito_client_secret": decrypted_credentials.0,
					"avito_client_id": decrypted_credentials.1,
					"is_connected": &acc.is_connected,
					"createdTs": &acc.created_ts,
					"updatedTs": &acc.updated_ts
				})
			}).collect::<Vec<_>>()
		})
	});

	HttpResponse::Ok().json(json_response)
}

#[get("/avito/accounts/{id}")]
pub async fn get_avito_account_by_id_handler(
	path: Path<Uuid>,
	data: web::Data<AppState>,
) -> impl Responder {
	let account_id = path.into_inner();

	let query_result = sqlx::query_as!(
        DbAvitoAccount,
        "SELECT account_id, user_id, client_id, avito_client_secret, avito_client_id, is_connected, created_ts, updated_ts FROM avito_accounts WHERE account_id = $1",
        account_id
    )
    .fetch_one(&data.db)
    .await;

	match query_result {
		Ok(db_account) => {
			let account: AvitoAccount = db_account.into();
			// Decrypt the credentials for the response
			let decrypted_credentials =
				decrypt_avito_credentials(&account.avito_client_secret, &account.avito_client_id)
					.unwrap_or_else(|_| (String::new(), String::new()));

			let json_response = json!({
				"status": "success",
				"data": json!({
					"avito_account": {
						"account_id": account.account_id,
						"user_id": &account.user_id,
						"client_id": &account.client_id,
						"avito_client_secret": decrypted_credentials.0,
						"avito_client_id": decrypted_credentials.1,
						"is_connected": &account.is_connected,
						"createdTs": &account.created_ts,
						"updatedTs": &account.updated_ts
					}
				})
			});

			HttpResponse::Ok().json(json_response)
		}
		Err(_) => {
			let message = "Avito account not found";
			HttpResponse::NotFound().json(json!({"status": "error", "message": message}))
		}
	}
}

#[post("/avito/accounts")]
pub async fn create_avito_account_handler(
	opts: web::Json<CreateAvitoAccountSchema>,
	data: web::Data<AppState>,
) -> impl Responder {
	let user_id = &opts.user_id;
	let avito_client_secret = &opts.avito_client_secret;
	let avito_client_id = &opts.avito_client_id;
	let is_connected = opts.is_connected.unwrap_or(false);

	// Get token from Avito API using the provided credentials
	let avito_token =
		match get_avito_token(&avito_client_id, &avito_client_secret, "client_credentials").await {
			Ok(token) => token,
			Err(_) => {
				return HttpResponse::BadRequest().json(
					json!({"status": "error", "message": "Failed to get token from Avito API"}),
				);
			}
		};

	// Get client_id from Avito API using the obtained token
	let client_id = match get_client_id_from_avito(&avito_token).await {
		Ok(id) => id.to_string(),
		Err(_) => {
			return HttpResponse::BadRequest().json(
				json!({"status": "error", "message": "Failed to get client_id from Avito API"}),
			);
		}
	};

	// Encrypt the sensitive data with IV
	let iv_secret = generate_iv();
	let encrypted_secret = encrypt_data(avito_client_secret, &ENCRYPTION_KEY, &iv_secret);
	let combined_secret = combine_iv_and_data(&iv_secret, &encrypted_secret);

	let iv_client_id_field = generate_iv();
	let encrypted_client_id = encrypt_data(avito_client_id, &ENCRYPTION_KEY, &iv_client_id_field);
	let combined_client_id = combine_iv_and_data(&iv_client_id_field, &encrypted_client_id);

	let query_result = sqlx::query_as!(
        DbAvitoAccount,
        r#"
        INSERT INTO avito_accounts (user_id, client_id, avito_client_secret, avito_client_id, is_connected)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING account_id, user_id, client_id, avito_client_secret, avito_client_id, is_connected, created_ts, updated_ts
        "#,
        user_id,
        client_id,
        combined_secret,
        combined_client_id,
        is_connected
    )
    .fetch_one(&data.db)
    .await;

	match query_result {
		Ok(db_account) => {
			let account: AvitoAccount = db_account.into();
			// Decrypt the credentials for the response
			let decrypted_credentials =
				decrypt_avito_credentials(&account.avito_client_secret, &account.avito_client_id)
					.unwrap_or_else(|_| (String::new(), String::new()));

			let json_response = json!({
				"status": "success",
				"data": json!({
					"avito_account": {
						"account_id": account.account_id,
						"user_id": &account.user_id,
						"client_id": &account.client_id,
						"avito_client_secret": decrypted_credentials.0,
						"avito_client_id": decrypted_credentials.1,
						"is_connected": &account.is_connected,
						"createdTs": &account.created_ts,
						"updatedTs": &account.updated_ts
					}
				})
			});
			HttpResponse::Ok().json(json_response)
		}
		Err(e) => {
			let message = format!("Что-то пошло не так при создании аккаунта Avito: {}", e);
			HttpResponse::InternalServerError().json(json!({"status": "error", "message": message}))
		}
	}
}

#[put("/avito/accounts/{id}")]
pub async fn update_avito_account_handler(
	path: Path<Uuid>,
	opts: web::Json<UpdateAvitoAccountSchema>,
	data: web::Data<AppState>,
) -> impl Responder {
	let account_id = path.into_inner();

	// Get the current account to check existing values
	let existing_db_account = sqlx::query_as!(
        DbAvitoAccount,
        "SELECT account_id, user_id, client_id, avito_client_secret, avito_client_id, is_connected, created_ts, updated_ts FROM avito_accounts WHERE account_id = $1",
        account_id
    )
    .fetch_one(&data.db)
    .await;

	let existing_account: AvitoAccount = match existing_db_account {
		Ok(db_account) => {
			db_account.into() // Convert DbAvitoAccount to AvitoAccount
		}
		Err(_) => {
			return HttpResponse::NotFound()
				.json(json!({"status": "error", "message": "Avito account not found"}));
		}
	};

	// Use existing values or new values if provided
	let user_id = opts.user_id.as_ref().unwrap_or(&existing_account.user_id);
	// For client_id, we use the existing one if not provided in the update
	let client_id = opts
		.client_id
		.as_ref()
		.unwrap_or(&existing_account.client_id);
	let is_connected = opts
		.is_connected
		.unwrap_or_else(|| existing_account.is_connected.unwrap_or(false));

	// Handle encryption for sensitive data if provided
	let avito_client_secret = if let Some(new_secret) = &opts.avito_client_secret {
		let iv_secret = generate_iv();
		let encrypted_secret = encrypt_data(new_secret, &ENCRYPTION_KEY, &iv_secret);
		combine_iv_and_data(&iv_secret, &encrypted_secret)
	} else {
		existing_account.avito_client_secret
	};

	let avito_client_id = if let Some(new_client_id) = &opts.avito_client_id {
		let iv_client_id = generate_iv();
		let encrypted_client_id = encrypt_data(new_client_id, &ENCRYPTION_KEY, &iv_client_id);
		combine_iv_and_data(&iv_client_id, &encrypted_client_id)
	} else {
		existing_account.avito_client_id
	};

	let query_result = sqlx::query_as!(
        DbAvitoAccount,
        r#"
        UPDATE avito_accounts
        SET user_id = $1, client_id = $2, avito_client_secret = $3, avito_client_id = $4, is_connected = $5, updated_ts = $6
        WHERE account_id = $7
        RETURNING account_id, user_id, client_id, avito_client_secret, avito_client_id, is_connected, created_ts, updated_ts
        "#,
        user_id,
        client_id,
        avito_client_secret,
        avito_client_id,
        is_connected,
        chrono::Utc::now().naive_utc(), // Convert to NaiveDateTime for the database
        account_id
    )
    .fetch_one(&data.db)
    .await;

	match query_result {
		Ok(db_account) => {
			let account: AvitoAccount = db_account.into();
			// Decrypt the credentials for the response
			let decrypted_credentials =
				decrypt_avito_credentials(&account.avito_client_secret, &account.avito_client_id)
					.unwrap_or_else(|_| (String::new(), String::new()));

			let json_response = json!({
				"status": "success",
				"data": json!({
					"avito_account": {
						"account_id": account.account_id,
						"user_id": &account.user_id,
						"client_id": &account.client_id,
						"avito_client_secret": decrypted_credentials.0,
						"avito_client_id": decrypted_credentials.1,
						"is_connected": &account.is_connected,
						"createdTs": &account.created_ts,
						"updatedTs": &account.updated_ts
					}
				})
			});
			HttpResponse::Ok().json(json_response)
		}
		Err(e) => {
			let message = format!("Что-то пошло не так при обновлении аккаунта Avito: {}", e);
			HttpResponse::InternalServerError().json(json!({"status": "error", "message": message}))
		}
	}
}

#[delete("/avito/accounts/{id}")]
pub async fn delete_avito_account_handler(
	path: Path<Uuid>,
	data: web::Data<AppState>,
) -> impl Responder {
	let account_id = path.into_inner();

	let query_result = sqlx::query!(
		"DELETE FROM avito_accounts WHERE account_id = $1",
		account_id
	)
	.execute(&data.db)
	.await;

	match query_result {
		Ok(result) => {
			if result.rows_affected() == 0 {
				return HttpResponse::NotFound()
					.json(json!({"status": "error", "message": "Avito account not found"}));
			}

			let json_response = json!({
				"status": "success",
				"message": "Avito account deleted successfully"
			});
			HttpResponse::Ok().json(json_response)
		}
		Err(e) => {
			let message = format!("Что-то пошло не так при удалении аккаунта Avito: {}", e);
			HttpResponse::InternalServerError().json(json!({"status": "error", "message": message}))
		}
	}
}

// Function to decrypt data when needed (for internal use)
pub fn decrypt_avito_credentials(
	encrypted_secret: &str,
	encrypted_client_id: &str,
) -> Result<(String, String), Box<dyn std::error::Error>> {
	// Split IV and encrypted data for secret
	let (iv_secret, encrypted_secret_data) = split_iv_and_data(encrypted_secret)?;
	let secret = decrypt_data(&encrypted_secret_data, &ENCRYPTION_KEY, &iv_secret)?;

	// Split IV and encrypted data for client_id
	let (iv_client_id, encrypted_client_id_data) = split_iv_and_data(encrypted_client_id)?;
	let client_id = decrypt_data(&encrypted_client_id_data, &ENCRYPTION_KEY, &iv_client_id)?;

	Ok((secret, client_id))
}
