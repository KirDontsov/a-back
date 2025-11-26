use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt;

#[derive(Debug, Deserialize, Serialize, Clone)]
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

#[derive(Debug, Deserialize, Serialize)]
pub struct AvitoGetItemsApiResponse {
	pub meta: AvitoMeta,
	pub resources: Vec<AvitoResource>,
}

#[derive(Debug, Deserialize)]
pub struct GetAvitoItemsParams {
	pub avito_token: String,
	pub page: Option<usize>,
	pub per_page: Option<usize>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AvitoMeta {
	pub page: usize,
	pub per_page: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AvitoResource {
	pub address: String,
	pub category: AvitoCategory,
	pub id: usize,
	pub price: usize,
	pub status: String,
	pub title: String,
	pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AvitoCategory {
	pub id: usize,
	pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AvitoGetBalanceApiResponse {
	pub balance: usize,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct AvitoTokenParams {
	pub avito_token: String,
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

#[derive(Debug, Deserialize)]
pub struct GetItemAnalyticsBody {
	pub avito_token: String,
	pub account_id: String,
	#[serde(rename = "dateFrom")]
	pub date_from: String,
	#[serde(rename = "dateTo")]
	pub date_to: String,
	pub grouping: String,
	pub limit: usize,
	pub metrics: Vec<String>,
	pub offset: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AvitoItemAnalyticsResponse {
	pub result: AnalyticsResult,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyticsResult {
	#[serde(rename = "dataTotalCount")]
	pub data_total_count: usize,
	pub groupings: Vec<AnalyticsGrouping>,
	pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyticsGrouping {
	pub id: usize,
	pub metrics: Vec<AnalyticsMetric>,
	#[serde(rename = "type")]
	pub grouping_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyticsMetric {
	pub slug: String,
	pub value: MetricValue,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetricValue {
	Integer(u32),
	Float(f32),
	String(String),
}

#[derive(Debug, Deserialize)]
pub struct UpdatePriceBody {
	pub avito_token: String,
	pub item_id: String,
	pub price: usize,
}

// Define the XML structures
#[derive(Debug, Deserialize)]
pub struct AvitoFeedAds {
	#[serde(rename = "Ad")] // Changed from "Ads" to "Ad"
	pub ads: Vec<AvitoFeedAd>,
}

#[derive(Debug, Deserialize)]
pub struct AvitoFeedAd {
	#[serde(rename = "Id")]
	pub id: String,
	#[serde(rename = "Category")]
	pub category: String,
	#[serde(rename = "Title")]
	pub title: String,
	#[serde(rename = "Description")]
	pub description: Option<String>,
}




#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct AvitoEditorCategoryFieldsParams {
	pub avito_token: String,
	pub avito_slug: String,
}

// ============================= Error Handling ========================
// Define a custom error type
#[derive(Debug)]
pub enum ApiError {
	InternalServerError(String),
	ReqwestError(reqwest::Error),
	AvitoApiError(u16, String),
	JsonParseError(serde_json::Error, String),
	DatabaseError(sqlx::Error),
	Other(String),
}

// Implement Display for your error type
impl fmt::Display for ApiError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			ApiError::ReqwestError(e) => write!(f, "HTTP request error: {}", e),
			ApiError::InternalServerError(e) => write!(f, "Server respond with error: {}", e),
			ApiError::AvitoApiError(code, e) => write!(f, "Avito API error ({}): {}", code, e),
			ApiError::JsonParseError(e, text) => {
				write!(f, "JSON parse error: {} - Response text: {}", e, text)
			}
			ApiError::DatabaseError(e) => write!(f, "Database error: {}", e),
			ApiError::Other(e) => write!(f, "Other error: {}", e),
		}
	}
}

// Keep the ResponseError implementation the same:
impl actix_web::error::ResponseError for ApiError {
	fn error_response(&self) -> HttpResponse {
		match self {
			ApiError::InternalServerError(_) => HttpResponse::InternalServerError().json(json!({
				"status": "error",
				"message": "Server respond with error"
			})),
			ApiError::ReqwestError(_) => HttpResponse::BadGateway().json(json!({
				"status": "error",
				"message": "Failed to communicate with Avito API"
			})),
			ApiError::AvitoApiError(status_code, message) => {
				// For 429 status, return the same status code
				if *status_code == 429 {
					return HttpResponse::TooManyRequests().json(json!({
						"status": "error",
						"message": message
					}));
				}

				// For other errors, return BadRequest as before
				HttpResponse::BadRequest().json(json!({
					"status": "error",
					"message": message
				}))
			}
			ApiError::JsonParseError(_, _) => HttpResponse::InternalServerError().json(json!({
				"status": "error",
				"message": "Failed to parse API response"
			})),
			ApiError::DatabaseError(_) => HttpResponse::InternalServerError().json(json!({
				"status": "error",
				"message": "Database error occurred"
			})),
			ApiError::Other(_) => HttpResponse::InternalServerError().json(json!({
				"status": "error",
				"message": "An unexpected error occurred"
			})),
		}
	}
}

// Implement From trait for sqlx::Error
impl From<sqlx::Error> for ApiError {
	fn from(err: sqlx::Error) -> ApiError {
		ApiError::DatabaseError(err)
	}
}

// Implement From traits for convenient error conversion
impl From<reqwest::Error> for ApiError {
	fn from(err: reqwest::Error) -> ApiError {
		ApiError::ReqwestError(err)
	}
}

impl From<serde_json::Error> for ApiError {
	fn from(err: serde_json::Error) -> ApiError {
		ApiError::JsonParseError(err, String::new())
	}
}

#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct AvitoCarMark {
	pub car_mark_id: uuid::Uuid,
	pub value: String,
}
