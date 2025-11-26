use crate::controllers::auth::Role;
use crate::{
	jwt_auth::JwtMiddleware,
	models::{ApiError, AvitoCarMark, AvitoEditorCategoryFieldsParams, AvitoTokenParams},
	AppState,
};
use actix_web::{
	post,
	web::{self},
	HttpResponse,
};
use actix_web_grants::proc_macro::has_any_role;
use serde_json::json;
use std::env;

use reqwest::{
	header::{self, HeaderValue},
	Client,
};

#[post("/avito/get_categories_tree")]
#[has_any_role("Role::Admin", type = "Role")]
pub async fn get_avito_categories_tree(
	opts: web::Json<AvitoTokenParams>,
	_: JwtMiddleware,
) -> Result<HttpResponse, ApiError> {
	let avito_token = opts.avito_token.clone();

	let url = env::var("AVITO_BASE_URL")
		.map_err(|_| ApiError::Other("AVITO_BASE_URL not set".to_string()))?;

	// Build headers
	let mut headers = header::HeaderMap::new();
	headers.insert(
		header::AUTHORIZATION,
		format!("Bearer {}", avito_token).parse().unwrap(),
	);
	//    headers.insert(
	// 	HeaderName::from_static("If-Modified-Since"),
	// 	HeaderValue::from_static("Mon, 01 Aug 2025 00:00:00 UTC"),
	// );

	// Build URL for user docs tree endpoint
	let api_url = format!("{}/autoload/v1/user-docs/tree", url);

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
	let docs_tree_data: serde_json::Value = serde_json::from_str(&response_text)
		.map_err(|_e| ApiError::JsonParseError(_e, response_text.clone()))?;

	Ok(HttpResponse::Ok().json(json!({
		"status": "success",
		"data": docs_tree_data
	})))
}

#[post("/avito/get_category_fields")]
#[has_any_role("Role::Admin", type = "Role")]
pub async fn get_avito_category_fields(
	opts: web::Json<AvitoEditorCategoryFieldsParams>,
	_: JwtMiddleware,
	data: web::Data<AppState>,
) -> Result<HttpResponse, ApiError> {
	let avito_token = opts.avito_token.clone();
	let avito_slug = opts.avito_slug.clone();

	let url = env::var("AVITO_BASE_URL")
		.map_err(|_| ApiError::Other("AVITO_BASE_URL not set".to_string()))?;

	// Build headers
	let mut headers = header::HeaderMap::new();
	headers.insert(
		header::AUTHORIZATION,
		format!("Bearer {}", avito_token).parse().unwrap(),
	);
	headers.insert(header::ACCEPT, HeaderValue::from_static("application/json"));

	// Build URL for user docs node fields endpoint
	let api_url = format!("{}/autoload/v1/user-docs/node/{}/fields", url, avito_slug);

	// Make request
	let response = Client::builder()
		.danger_accept_invalid_certs(true)
		.build()?
		.get(&api_url)
		.headers(headers.clone())
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
	let mut node_fields_data: serde_json::Value = serde_json::from_str(&response_text)
		.map_err(|_e| ApiError::JsonParseError(_e, response_text.clone()))?;

	// Process the response to fetch additional data from values_link_json and values_link_xml
	if let Some(fields_array) = node_fields_data
		.get_mut("fields")
		.and_then(|f| f.as_array_mut())
	{
		for field in fields_array.iter_mut() {
			// Process content array of the main field
			if let Some(content_array) = field.get_mut("content").and_then(|c| c.as_array_mut()) {
				for content_item in content_array.iter_mut() {
					// Process values_link_json
					if let Some(values_link_json) = content_item
						.get("values_link_json")
						.and_then(|v| v.as_str())
					{
						// Make additional request to fetch values from the JSON link
						let values_response = Client::builder()
							.danger_accept_invalid_certs(true)
							.timeout(std::time::Duration::from_secs(5)) // Add 5 second timeout
							.build()?
							.get(values_link_json)
							.headers(headers.clone())
							.send()
							.await?;

						if values_response.status().is_success() {
							let values_text = values_response.text().await?;
							let values_data: serde_json::Value = serde_json::from_str(&values_text)
								.map_err(|e| ApiError::JsonParseError(e, values_text.clone()))?;

							// Add the fetched values to the content item as a new "values" field
							content_item
								.as_object_mut()
								.unwrap()
								.insert("values".to_string(), values_data);
						}
					} else if let Some(values_link_xml) =
						content_item.get("values_link_xml").and_then(|v| v.as_str())
					{
						// Check if the values_link_xml contains "Autocatalog.xml"
						if values_link_xml.contains("Autocatalog.xml") {
							// Fetch data from avito_car_marks table
							match sqlx::query_as::<_, AvitoCarMark>(
								"SELECT car_mark_id, value FROM avito_car_marks",
							)
							.fetch_all(&data.db)
							.await
							{
								Ok(car_marks) => {
									// Convert car marks to the expected JSON format
									let values_data =
										serde_json::to_value(&car_marks).map_err(|e| {
											ApiError::JsonParseError(
												e,
												format!(
													"Failed to serialize car marks: {:?}",
													car_marks
												),
											)
										})?;

									// Add the fetched values to the content item as a new "values" field
									content_item
										.as_object_mut()
										.unwrap()
										.insert("values".to_string(), values_data);
								}
								Err(_) => {
									// If database query fails (e.g., table doesn't exist),
									// fall back to original behavior of making HTTP request
									// This preserves functionality when database is not set up
								}
							}
						} else {
							// For other XML links, make HTTP request as before (if needed)
							// This preserves the original behavior for non-Autocatalog.xml links
						}
					}

					// Remove the values_link_json and values_link_xml fields since we've fetched the data
					content_item
						.as_object_mut()
						.unwrap()
						.remove("values_link_json");
					content_item
						.as_object_mut()
						.unwrap()
						.remove("values_link_xml");
				}
			}

			// Process children array if it exists
			if let Some(children_array) = field.get_mut("children").and_then(|c| c.as_array_mut()) {
				for child in children_array.iter_mut() {
					// Extract the tag value before mutable borrows to avoid borrowing conflicts
					let tag_value = child
						.get("tag")
						.and_then(|t| t.as_str())
						.map(|s| s.to_string());

					// Process content array of each child
					if let Some(child_content_array) =
						child.get_mut("content").and_then(|c| c.as_array_mut())
					{
						for child_content_item in child_content_array.iter_mut() {
							// Process values_link_json in children
							if let Some(values_link_json) = child_content_item
								.get("values_link_json")
								.and_then(|v| v.as_str())
							{
								// Make additional request to fetch values from the JSON link
								let values_response = Client::builder()
									.danger_accept_invalid_certs(true)
									.timeout(std::time::Duration::from_secs(5)) // Add 5 second timeout
									.build()?
									.get(values_link_json)
									.headers(headers.clone())
									.send()
									.await?;

								if values_response.status().is_success() {
									let values_text = values_response.text().await?;
									let values_data: serde_json::Value =
										serde_json::from_str(&values_text).map_err(|e| {
											ApiError::JsonParseError(e, values_text.clone())
										})?;

									// Add the fetched values to the content item as a new "values" field
									child_content_item
										.as_object_mut()
										.unwrap()
										.insert("values".to_string(), values_data);
								}
							} else if let Some(values_link_xml) = child_content_item
								.get("values_link_xml")
								.and_then(|v| v.as_str())
							{
								// Check if the values_link_xml contains "Autocatalog.xml"
								if values_link_xml.contains("Autocatalog.xml") {
									// Skip database query if the field's tag is one of the specified values
									let should_skip_db_query = if let Some(ref tag_val) = tag_value
									{
										tag_val == "Model"
											|| tag_val == "Generation" || tag_val == "Modification"
											|| tag_val == "BodyType" || tag_val == "Doors"
									} else {
										false
									};

									if !should_skip_db_query {
										// Fetch data from avito_car_marks table
										match sqlx::query_as::<_, AvitoCarMark>(
											"SELECT car_mark_id, value FROM avito_car_marks",
										)
										.fetch_all(&data.db)
										.await
										{
											Ok(car_marks) => {
												// Convert car marks to the expected JSON format
												let values_data = serde_json::to_value(&car_marks)
													.map_err(|e| {
														ApiError::JsonParseError(e, format!("Failed to serialize car marks: {:?}", car_marks))
													})?;

												// Add the fetched values to the content item as a new "values" field
												child_content_item.as_object_mut().unwrap().insert(
													"values".to_string(),
													values_data.clone(),
												); // Clone to avoid move
											}
											Err(_e) => {
												// If database query fails (e.g., table doesn't exist),
												// fall back to original behavior of making HTTP request
												// This preserves functionality when database is not set up
											}
										}
									}
								} else {
									// For other XML links, make HTTP request as before (if needed)
									// This preserves the original behavior for non-Autocatalog.xml links
								}
							}

							// Remove the values_link_json and values_link_xml fields since we've fetched the data
							child_content_item
								.as_object_mut()
								.unwrap()
								.remove("values_link_json");
							child_content_item
								.as_object_mut()
								.unwrap()
								.remove("values_link_xml");
						}
					}
				}
			}
		}
	}

	Ok(HttpResponse::Ok().json(json!({
		"status": "success",
		"data": node_fields_data
	})))
}
