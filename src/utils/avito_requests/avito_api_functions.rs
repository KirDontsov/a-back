use reqwest::{header::{self, HeaderMap, HeaderValue}};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct AvitoTokenCredentials {
	pub client_id: String,
	pub client_secret: String,
	pub grant_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AvitoTokenResponse {
	pub access_token: String,
	pub token_type: String,
	pub expires_in: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AvitoUserProfileResponse {
	pub id: usize,
	pub name: String,
	pub email: Option<String>,
	pub phone: Option<String>,
	pub phones: Option<Vec<String>>,
	pub profile_url: String,
}

// Function to get token from Avito API using client credentials
pub async fn get_avito_token(
	client_id: &str,
	client_secret: &str,
	grant_type: &str,
) -> Result<String, Box<dyn std::error::Error>> {
	let url = env::var("AVITO_BASE_URL")?;

	let mut headers = HeaderMap::new();
	headers.insert(
		header::CONTENT_TYPE,
		"application/x-www-form-urlencoded".parse().unwrap(),
	);

	let body = AvitoTokenCredentials {
		client_id: client_id.to_string(),
		client_secret: client_secret.to_string(),
		grant_type: grant_type.to_string(),
	};

	// Make request
	let response = reqwest::Client::builder()
		.danger_accept_invalid_certs(true)
		.build()?
		.post(format!("{}/token", url))
		.headers(headers)
		.form(&body)
		.send()
		.await?;

	// Check response status
	if !response.status().is_success() {
		let status_code = response.status().as_u16();
		let error_body = response.text().await?;
		return Err(format!("Avito API error ({}): {}", status_code, error_body).into());
	}

	// Parse response
	let response_text = response.text().await?;
	let token_data: AvitoTokenResponse = serde_json::from_str(&response_text)?;

	Ok(token_data.access_token)
}

// Function to get client_id from Avito API
pub async fn get_client_id_from_avito(
	avito_token: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
	let url = env::var("AVITO_BASE_URL")?;

	// Build headers
	let mut headers = HeaderMap::new();
	headers.insert(
		header::AUTHORIZATION,
		format!("Bearer {}", avito_token).parse().unwrap(),
	);
	headers.insert(header::USER_AGENT, HeaderValue::from_static("YourApp/1.0"));
	headers.insert(header::ACCEPT, HeaderValue::from_static("application/json"));

	// Build URL
	let api_url = format!("{}/core/v1/accounts/self", url);

	// Make request
	let response = reqwest::Client::builder()
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
		return Err(format!("Avito API error ({}): {}", status_code, error_body).into());
	}

	// Parse response
	let response_text = response.text().await?;
	let profile_data: AvitoUserProfileResponse = serde_json::from_str(&response_text)?;

	Ok(profile_data.id)
}
