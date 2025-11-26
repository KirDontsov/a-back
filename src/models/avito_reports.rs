use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct AvitoReportsResponse {
	pub reports: Vec<AvitoReport>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AvitoReport {
	pub id: i64,
	#[serde(rename = "started_at")]
	pub started_at: String,
	#[serde(rename = "finished_at")]
	pub finished_at: String,
	pub status: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AvitoReportItemsResponse {
	#[serde(rename = "report_id")]
	pub report_id: i64,
	pub meta: AvitoReportItemsMeta,
	pub items: Vec<AvitoReportItem>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AvitoReportItemsMeta {
	#[serde(rename = "per_page")]
	pub per_page: i64,
	pub page: i64,
	pub pages: i64,
	pub total: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AvitoReportItem {
	pub section: AvitoReportItemSection,
	#[serde(rename = "ad_id")]
	pub ad_id: String,
	#[serde(rename = "avito_id")]
	pub avito_id: i64,
	#[serde(rename = "feed_name")]
	pub feed_name: String,
	pub url: String,
	pub messages: Vec<AvitoReportItemMessage>,
	#[serde(rename = "avito_date_end")]
	pub avito_date_end: String,
	#[serde(rename = "avito_status")]
	pub avito_status: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AvitoReportItemSection {
	pub slug: String,
	pub title: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AvitoReportItemMessage {
	pub code: i64,
	pub title: String,
	pub description: String,
	#[serde(rename = "type")]
	pub message_type: String,
	#[serde(rename = "updated_at")]
	pub updated_at: String,
}
