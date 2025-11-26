use crate::controllers::auth::Role;
use crate::{
	jwt_auth::JwtMiddleware,
	models::{
		ApiError, AvitoReport, AvitoReportItem, AvitoReportItemsResponse, AvitoReportsResponse,
	},
	AppState,
};
use actix_web::{
	post,
	web::{self},
	HttpResponse,
};
use actix_web_grants::proc_macro::has_any_role;
use reqwest::{
	header::{self, HeaderMap, HeaderValue},
	Client,
};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct AvitoTokenParams {
	pub avito_token: String,
}

// Function to fetch Avito reports
async fn fetch_avito_reports(avito_token: &str) -> Result<AvitoReportsResponse, ApiError> {
	let url = env::var("AVITO_BASE_URL")
		.map_err(|_| ApiError::Other("AVITO_BASE_URL not set".to_string()))?;

	let mut headers = HeaderMap::new();
	headers.insert(
		header::AUTHORIZATION,
		format!("Bearer {}", avito_token).parse().unwrap(),
	);
	headers.insert(header::ACCEPT, HeaderValue::from_static("application/json"));

	let api_url = format!("{}/autoload/v2/reports", url);

	let response = Client::builder()
		.danger_accept_invalid_certs(true)
		.build()?
		.get(&api_url)
		.headers(headers)
		.send()
		.await?;

	if !response.status().is_success() {
		let status_code = response.status().as_u16();
		let error_body = response.text().await?;
		return Err(ApiError::AvitoApiError(status_code, error_body));
	}

	let response_text = response.text().await?;
	let reports_data: AvitoReportsResponse = serde_json::from_str(&response_text)
		.map_err(|e| ApiError::JsonParseError(e, response_text.clone()))?;

	Ok(reports_data)
}

// Function to fetch items for a specific report with pagination
async fn fetch_report_items(
	avito_token: &str,
	report_id: i64,
	page: i64,
	per_page: i64,
) -> Result<AvitoReportItemsResponse, ApiError> {
	let url = env::var("AVITO_BASE_URL")
		.map_err(|_| ApiError::Other("AVITO_BASE_URL not set".to_string()))?;

	let mut headers = HeaderMap::new();
	headers.insert(
		header::AUTHORIZATION,
		format!("Bearer {}", avito_token).parse().unwrap(),
	);
	headers.insert(header::ACCEPT, HeaderValue::from_static("application/json"));

	let api_url = format!(
		"{}/autoload/v2/reports/{}/items?page={}&per_page={}",
		url, report_id, page, per_page
	);

	let response = Client::builder()
		.danger_accept_invalid_certs(true)
		.build()?
		.get(&api_url)
		.headers(headers)
		.send()
		.await?;

	if !response.status().is_success() {
		let status_code = response.status().as_u16();
		let error_body = response.text().await?;
		return Err(ApiError::AvitoApiError(status_code, error_body));
	}

	let response_text = response.text().await?;
	let items_data: AvitoReportItemsResponse = serde_json::from_str(&response_text)
		.map_err(|e| ApiError::JsonParseError(e, response_text.clone()))?;

	Ok(items_data)
}

// Function to get the latest report by finished_at timestamp
fn get_latest_report(reports: Vec<AvitoReport>) -> Option<AvitoReport> {
	reports
		.into_iter()
		.filter(|report| report.status == "success")
		.max_by(|a, b| a.finished_at.cmp(&b.finished_at))
}

// Function to fetch all items from a report handling pagination
async fn fetch_all_report_items(
	avito_token: &str,
	report_id: i64,
) -> Result<Vec<AvitoReportItem>, ApiError> {
	let per_page = 200;
	let mut all_items = Vec::new();
	let mut page = 0;

	loop {
		let items_response = fetch_report_items(avito_token, report_id, page, per_page).await?;

		all_items.extend(items_response.items);

		// Check if we've fetched all items
		if all_items.len() >= items_response.meta.total as usize {
			break;
		}

		page += 1;
	}

	Ok(all_items)
}

// Function to update avito_ads table with parsed_id and avito_ad_id mapping
async fn update_avito_ads_table(
	data: &web::Data<AppState>,
	items: &Vec<AvitoReportItem>,
) -> Result<(), ApiError> {
	// Create vectors for batch insert
	let mut parsed_ids = Vec::new();
	let mut avito_ids = Vec::new();

	for item in items {
		parsed_ids.push(item.ad_id.clone());
		avito_ids.push(item.avito_id.to_string());
	}

	// Update the avito_ads table with the avito_id for each parsed_id
	// Using a batch update approach
	for (parsed_id, avito_id) in parsed_ids.iter().zip(avito_ids.iter()) {
		sqlx::query!(
			r#"
            UPDATE avito_ads 
            SET avito_ad_id = $1 
            WHERE parsed_id = $2
            "#,
			avito_id,
			parsed_id
		)
		.execute(&data.db)
		.await
		.map_err(|e| ApiError::InternalServerError(format!("Failed to update avito_ads: {}", e)))?;
	}

	Ok(())
}

#[post("/avito/fetch-and-update-ads")]
#[has_any_role("Role::Admin", type = "Role")]
pub async fn fetch_and_update_avito_ads(
	opts: web::Json<AvitoTokenParams>,
	data: web::Data<AppState>,
	_: JwtMiddleware,
) -> Result<HttpResponse, ApiError> {
	let avito_token = opts.avito_token.clone();

	// Fetch reports
	let reports_response = fetch_avito_reports(&avito_token).await?;

	// Get the latest report
	let latest_report = match get_latest_report(reports_response.reports) {
		Some(report) => report,
		None => return Err(ApiError::Other("No successful reports found".to_string())),
	};

	// Fetch all items from the latest report
	let all_items = fetch_all_report_items(&avito_token, latest_report.id).await?;

	// Update the avito_ads table
	update_avito_ads_table(&data, &all_items).await?;

	Ok(HttpResponse::Ok().json(serde_json::json!({
		"status": "success",
		"message": "Avito ads updated successfully",
		"report_id": latest_report.id,
		"items_processed": &all_items.len()
	})))
}
