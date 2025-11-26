use crate::controllers::auth::Role;
use crate::utils::avito_requests::{
	get_avito_token as shared_get_avito_token,
	get_client_id_from_avito as shared_get_client_id_from_avito,
};
use crate::{
	jwt_auth::JwtMiddleware,
	models::{
		ApiError, AvitoGetBalanceApiResponse, AvitoGetItemsApiResponse, AvitoItemAnalyticsResponse,
		AvitoTokenCredentials, AvitoTokenParams, GetAvitoItemsParams, GetItemAnalyticsBody,
		UpdatePriceBody,
	},
};
use actix_web::{
	cookie::{time::Duration as ActixWebDuration, Cookie, SameSite},
	post,
	web::{self},
	HttpResponse,
};
use actix_web_grants::proc_macro::has_any_role;
use serde_json::json;
use std::env;

use reqwest::{
	header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE, USER_AGENT},
	Client,
};

#[post("/avito/get_token")]
#[has_any_role("Role::Admin", type = "Role")]
pub async fn get_avito_token_handler(
	credentials: web::Json<AvitoTokenCredentials>,
	_: JwtMiddleware,
) -> Result<HttpResponse, ApiError> {
	let client_id = credentials.client_id.clone();
	let client_secret = credentials.client_secret.clone();
	let grant_type = credentials.grant_type.clone();

	// Use the shared function to get the token
	let access_token = shared_get_avito_token(&client_id, &client_secret, &grant_type)
		.await
		.map_err(|e| ApiError::Other(format!("Failed to get token: {}", e)))?;

	// Build cookie
	let cookie = Cookie::build("avito_token", &access_token)
		.same_site(SameSite::None)
		.path("/")
		.max_age(ActixWebDuration::new(3600, 0)) // Default expiration of 1 hour
		.secure(true)
		.finish();

	Ok(HttpResponse::Ok().cookie(cookie).json(json!({
		"status": "success",
		"data": {
			"access_token": access_token,
			"token_type": "Bearer",
			"expires_in": 3600,
		}
	})))
}

#[post("/avito/get_items")]
#[has_any_role("Role::Admin", type = "Role")]
pub async fn get_avito_items(
	opts: web::Json<GetAvitoItemsParams>,
	_: JwtMiddleware,
) -> Result<HttpResponse, ApiError> {
	let avito_token = opts.avito_token.clone();
	let page = opts.page.unwrap_or(0);
	let per_page = opts.per_page.unwrap_or(50).min(1000); // Avito API max per_page is 1000

	let url = env::var("AVITO_BASE_URL")
		.map_err(|_| ApiError::Other("AVITO_BASE_URL not set".to_string()))?;

	let mut headers = HeaderMap::new();
	headers.insert(
		CONTENT_TYPE,
		"application/x-www-form-urlencoded".parse().unwrap(),
	);
	headers.insert(
		AUTHORIZATION,
		format!("Bearer {}", avito_token).parse().unwrap(),
	);

	// Build URL with pagination parameters
	let api_url = format!("{}/core/v1/items?page={}&per_page={}", url, page, per_page);

	// Make request
	let response = Client::builder()
		.danger_accept_invalid_certs(true)
		.build()?
		.get(&api_url)
		.headers(headers)
		.send()
		.await?;

	// Check response status
	if !response.status().is_success() {
		let status_code = response.status().as_u16();
		let error_body = response.text().await?;
		return Err(ApiError::AvitoApiError(status_code, error_body));
	}

	// Parse response
	let response_text = response.text().await?;
	let respon_data: AvitoGetItemsApiResponse = serde_json::from_str(&response_text)
		.map_err(|e| ApiError::JsonParseError(e, response_text.clone()))?;

	Ok(HttpResponse::Ok().json(json!({
		"status": "success",
		"data": {
			"meta": &respon_data.meta,
			"items": &respon_data.resources,
		},
	})))
}

#[post("/avito/get_balance")]
#[has_any_role("Role::Admin", type = "Role")]
pub async fn get_avito_balance(
	opts: web::Json<AvitoTokenParams>,
	_: JwtMiddleware,
) -> Result<HttpResponse, ApiError> {
	let avito_token = opts.avito_token.clone();

	let url = env::var("AVITO_BASE_URL").expect("AVITO_BASE_URL not set");

	let headers: HeaderMap<HeaderValue> = HeaderMap::from_iter(vec![
		(CONTENT_TYPE, "application/json".parse().unwrap()),
		(
			AUTHORIZATION,
			format!("Bearer {}", avito_token).parse().unwrap(),
		),
	]);

	let body = json!({});

	let response = Client::builder()
		.danger_accept_invalid_certs(true)
		.build()?
		.post(format!("{}/cpa/v3/balanceInfo", url))
		.headers(headers)
		.json(&body)
		.send()
		.await?;

	// Check response status
	if !response.status().is_success() {
		let status_code = response.status().as_u16();
		let error_body = response.text().await?;
		return Err(ApiError::AvitoApiError(status_code, error_body));
	}

	let response_text: String = response.text().await?;

	// Parse the response with error context
	let respon_data: AvitoGetBalanceApiResponse = serde_json::from_str(&response_text)
		.map_err(|e| ApiError::JsonParseError(e, response_text.to_string()))?;

	Ok(HttpResponse::Ok().json(json!({
		"status": "success",
		"data": {
			"balance": &respon_data.balance,
		}
	})))
}

#[post("/avito/get_user_profile")]
#[has_any_role("Role::Admin", type = "Role")]
pub async fn get_avito_user_profile(
	opts: web::Json<AvitoTokenParams>,
	_: JwtMiddleware,
) -> Result<HttpResponse, ApiError> {
	let avito_token = opts.avito_token.clone();

	// Use the shared function to get the client ID from the token
	let client_id = shared_get_client_id_from_avito(&avito_token)
		.await
		.map_err(|e| ApiError::Other(format!("Failed to get client profile: {}", e)))?;

	// Create a response similar to the original AvitoUserProfileResponse
	// Since we only get the ID from the shared function, we'll return a minimal response
	Ok(HttpResponse::Ok().json(json!({
	"status": "success",
	"data": {
			"id": client_id,
			"name": "User Profile", // Placeholder since we only get ID from the shared function
			"email": null,
			"phone": null,
			"phones": null,
			"profile_url": format!("https://www.avito.ru/user/{}", client_id)
		}
	})))
}

#[post("/avito/get_item_analytics")]
pub async fn get_avito_item_analytics(
	opts: web::Json<GetItemAnalyticsBody>,
	_: JwtMiddleware,
) -> Result<HttpResponse, ApiError> {
	let avito_token = opts.avito_token.clone();
	let account_id = opts.account_id.clone();

	let url = env::var("AVITO_BASE_URL")
		.map_err(|_| ApiError::Other("AVITO_BASE_URL not set".to_string()))?;

	// Build headers
	let mut headers = HeaderMap::new();
	headers.insert(
		AUTHORIZATION,
		format!("Bearer {}", avito_token).parse().unwrap(),
	);
	headers.insert(USER_AGENT, HeaderValue::from_static("YourApp/1.0"));
	headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
	headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

	// Build request body
	let request_body = json!({
		"dateFrom": opts.date_from,
		"dateTo": opts.date_to,
		"grouping": opts.grouping,
		"limit": opts.limit,
		"metrics": opts.metrics,
		"offset": opts.offset
	});

	// Build URL
	let api_url = format!("{}/stats/v2/accounts/{}/items", url, account_id);

	// Make request
	let response = Client::builder()
		.danger_accept_invalid_certs(true)
		.build()?
		.post(&api_url)
		.headers(headers)
		.json(&request_body)
		.send()
		.await?;

	// Check response status
	if !response.status().is_success() {
		let status_code = response.status().as_u16();
		let error_body = response.text().await?;
		return Err(ApiError::AvitoApiError(status_code, error_body));
	}

	// Parse response
	let response_text: String = response.text().await?;

	let analytics_data: AvitoItemAnalyticsResponse = serde_json::from_str(&response_text)
		.map_err(|e| ApiError::JsonParseError(e, response_text.to_string()))?;

	Ok(HttpResponse::Ok().json(json!({
		"status": "success",
		"data": analytics_data.result
	})))
}

#[post("/avito/update_price")]
pub async fn update_avito_price(
	opts: web::Json<UpdatePriceBody>,
	_: JwtMiddleware,
) -> Result<HttpResponse, ApiError> {
	let avito_token = opts.avito_token.clone();
	let item_id = opts.item_id.clone();

	let url = env::var("AVITO_BASE_URL")
		.map_err(|_| ApiError::Other("AVITO_BASE_URL not set".to_string()))?;

	// Build headers
	let mut headers = HeaderMap::new();
	headers.insert(
		AUTHORIZATION,
		format!("Bearer {}", avito_token).parse().unwrap(),
	);
	headers.insert(USER_AGENT, HeaderValue::from_static("YourApp/1.0"));
	headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
	headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

	// Build request body
	let request_body = json!({
		"price": opts.price
	});

	// Build URL
	let api_url = format!("{}/core/v1/items/{}/update_price", url, item_id);

	// Make request
	let response = Client::builder()
		.danger_accept_invalid_certs(true)
		.build()?
		.post(&api_url)
		.headers(headers)
		.json(&request_body)
		.send()
		.await?;

	// Check response status
	if !response.status().is_success() {
		let status_code = response.status().as_u16();
		let error_body = response.text().await?;
		return Err(ApiError::AvitoApiError(status_code, error_body));
	}

	// Parse response
	let response_text: String = response.text().await?;

	let update_price_data: AvitoItemAnalyticsResponse = serde_json::from_str(&response_text)
		.map_err(|e| ApiError::JsonParseError(e, response_text.to_string()))?;

	Ok(HttpResponse::Ok().json(json!({
		"status": "success",
		"data": update_price_data.result
	})))
}
