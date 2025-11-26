use crate::{
	jwt_auth::JwtMiddleware,
	models::{ApiError, XmlAd},
	AppState,
};
use actix_web::{
	post,
	web::{self},
	HttpResponse,
};
use serde::Deserialize;
use uuid::Uuid;

use quick_xml::events::Event;
use quick_xml::Reader;
use reqwest::Client;
use sqlx::Row;
use sqlx::{Postgres, Transaction};
use std::collections::HashMap;
use std::time::Duration;

// Structure for POST request body containing account_id and xml_url
#[derive(Deserialize)]
pub struct ImportAvitoXmlRequest {
    pub account_id: Uuid,
    pub xml_url: String,
}

#[post("/avito/import-xml")]
pub async fn import_avito_xml(
	body: web::Json<ImportAvitoXmlRequest>,
	data: web::Data<AppState>,
	_: JwtMiddleware,
) -> Result<HttpResponse, ApiError> {
	let xml_url = &body.xml_url;
	let account_id = if Some(body.account_id).is_some() { body.account_id } else { Uuid::parse_str("2acc3808-15f1-4abb-b15e-c7f4780a87da").unwrap() };

	// Fetch XML data
	let client = Client::builder()
		.timeout(Duration::from_secs(30))
		.build()
		.map_err(|e| ApiError::InternalServerError(e.to_string()))?;

	let response = client
		.get(xml_url)
		.send()
		.await
		.map_err(|e| ApiError::InternalServerError(format!("Failed to fetch XML: {}", e)))?;

	if !response.status().is_success() {
		return Err(ApiError::InternalServerError(format!(
			"Failed to fetch XML: Status {}",
			response.status()
		)));
	}

	let xml_data = response
		.text()
		.await
		.map_err(|e| ApiError::InternalServerError(format!("Failed to read response: {}", e)))?;

	// Parse XML and extract ads
	println!("Parsing XML data with length: {}", xml_data.len());
	let ads = parse_xml_ads(&xml_data)?;
	println!("Parsed {} ads from XML", ads.len());

	// Print debug information about the first few ads
	for (i, ad) in ads.iter().take(3).enumerate() {
	println!("Ad {}: id={}, fields={}", i, ad.id, ad.fields.len());
		if let Some(images) = ad.fields.get("Images") {
			println!("  Images field: {}", images);
		} else {
			println!("  No Images field found");
		}
	}

	println!("Parsed {} ads from XML", ads.len());

	// Start transaction
	let mut tx = data.db.begin().await.map_err(|e| {
		ApiError::InternalServerError(format!("Failed to start transaction: {}", e))
	})?;

	// Create feed entry
	let feed_id = Uuid::new_v4();
	sqlx::query!(
		r#"
        INSERT INTO avito_feeds (feed_id, account_id, category)
        VALUES ($1, $2, $3)
        "#,
		feed_id,
		account_id,
		"IMPORT"
	)
	.execute(&mut *tx)
	.await
	.map_err(|e| ApiError::InternalServerError(format!("Failed to create feed: {}", e)))?;

	// Batch process ads
	println!("Starting batch processing of {} ads", ads.len());
	batch_process_ads(&mut tx, &feed_id, &ads).await?;
	println!("Finished batch processing");

	tx.commit().await.map_err(|e| {
		ApiError::InternalServerError(format!("Failed to commit transaction: {}", e))
	})?;

	Ok(HttpResponse::Ok().json(serde_json::json!({
		"status": "success",
		"message": "Import completed successfully",
		"feed_id": feed_id,
		"ads_processed": ads.len()
	})))
}

// Batch process ads for better performance
async fn batch_process_ads(
	tx: &mut Transaction<'_, Postgres>,
	feed_id: &Uuid,
	ads: &[XmlAd],
) -> Result<(), ApiError> {
	let mut ad_ids = Vec::new();
	let mut parsed_ids = Vec::new();
	let mut feed_ids = Vec::new();
	let mut is_active_flags = Vec::new();
	let mut statuses = Vec::new();

	// Prepare batch data for ads
	for ad in ads {
		let ad_id = Uuid::new_v4();
		ad_ids.push(ad_id);
		parsed_ids.push(ad.id.clone()); // Clone the String
		feed_ids.push(feed_id.clone()); // Clone the Uuid
		is_active_flags.push(true);
		statuses.push("active".to_string());
	}

	// Batch insert ads
	sqlx::query!(
		r#"
        INSERT INTO avito_ads (ad_id, parsed_id, feed_id, is_active, status)
        SELECT * FROM UNNEST(
            $1::uuid[],
            $2::varchar[],
            $3::uuid[],
            $4::bool[],
            $5::varchar[]
        )
        "#,
		&ad_ids,
		&parsed_ids,
		&feed_ids,
		&is_active_flags,
		&statuses,
	)
	.execute(&mut **tx)
	.await
	.map_err(|e| ApiError::InternalServerError(format!("Failed to batch insert ads: {}", e)))?;

	println!("Inserted {} ads in batch", ads.len());

	// Batch process fields
	let mut field_ids = Vec::new();
	let mut field_ad_ids = Vec::new();
	let mut field_tags = Vec::new();
	let mut field_data_types = Vec::new();
	let mut field_field_types = Vec::new();

	let mut field_value_ids = Vec::new();
	let mut field_value_field_ids = Vec::new();
	let mut field_values = Vec::new();

	for (i, ad) in ads.iter().enumerate() {
		let ad_id = ad_ids[i];

		for (tag, value) in &ad.fields {
			// Skip Id field and other empty fields, but allow empty Images field
			if tag == "Id" || value.trim().is_empty() {
				continue;
			}

			let field_id = Uuid::new_v4();
			field_ids.push(field_id);
			field_ad_ids.push(ad_id);
			field_tags.push(tag.clone()); // Clone the String
			field_data_types.push("string".to_string());
			field_field_types.push("attribute".to_string());

			let field_value_id = Uuid::new_v4();
			field_value_ids.push(field_value_id);
			field_value_field_ids.push(field_id);
			field_values.push(value.clone()); // Clone the String
		}
	}

	// Batch insert fields
	if !field_ids.is_empty() {
		sqlx::query!(
			r#"
            INSERT INTO avito_ad_fields (field_id, ad_id, tag, data_type, field_type)
            SELECT * FROM UNNEST(
                $1::uuid[],
                $2::uuid[],
                $3::varchar[],
                $4::varchar[],
                $5::varchar[]
            )
            "#,
			&field_ids,
			&field_ad_ids,
			&field_tags,
			&field_data_types,
			&field_field_types,
		)
		.execute(&mut **tx)
		.await
		.map_err(|e| {
			ApiError::InternalServerError(format!("Failed to batch insert fields: {}", e))
		})?;

		println!("Inserted {} fields in batch", field_ids.len());
	}

	// Batch insert field values
	if !field_value_ids.is_empty() {
		sqlx::query!(
			r#"
            INSERT INTO avito_ad_field_values (field_value_id, field_id, value)
            SELECT * FROM UNNEST(
                $1::uuid[],
                $2::uuid[],
                $3::varchar[]
            )
            "#,
			&field_value_ids,
			&field_value_field_ids,
			&field_values,
		)
		.execute(&mut **tx)
		.await
		.map_err(|e| {
			ApiError::InternalServerError(format!("Failed to batch insert field values: {}", e))
		})?;

		println!("Inserted {} field values in batch", field_value_ids.len());
	}

	Ok(())
}

pub fn parse_xml_ads(xml_data: &str) -> Result<Vec<XmlAd>, ApiError> {
	let mut reader = Reader::from_str(xml_data);
	let mut ads = Vec::new();
	let mut buf = Vec::new();
	let mut current_ad: Option<XmlAd> = None;
	let mut current_path = Vec::new();
	let mut current_values = String::new();
	let mut in_ad = false;
	let mut delivery_buffer = Vec::new();
	let mut images_buffer: Vec<String> = Vec::new();

	loop {
		match reader.read_event_into(&mut buf) {
			Ok(Event::Start(e)) => {
				let name = std::str::from_utf8(e.name().as_ref())
					.map_err(|e| ApiError::Other(format!("UTF-8 error: {}", e)))?
					.to_string();

				// println!("Start element: {}, current path: {:?}", name, current_path);
				current_path.push(name.clone());

				if name == "Ad" {
					in_ad = true;
					current_ad = Some(XmlAd {
						id: String::new(),
						fields: HashMap::new(),
					});
					delivery_buffer.clear();
					images_buffer.clear();
				}

				current_values.clear();
			}
			Ok(Event::Text(e)) => {
				// Extract text content directly from the bytes
				let text = std::str::from_utf8(e.into_inner().as_ref())
					.map_err(|e| ApiError::Other(format!("UTF-8 error: {}", e)))?
					.to_string();

				if in_ad && !&text.trim().is_empty() {
					current_values.push_str(&text);
				}
			}
			Ok(Event::CData(e)) => {
				// Handle CDATA content (for Description)
				let text = std::str::from_utf8(e.as_ref())
					.map_err(|e| ApiError::Other(format!("UTF-8 error: {}", e)))?
					.to_string();

				if in_ad {
					current_values.push_str(&text);
				}
			}
			Ok(Event::Empty(e)) => {
				let name = std::str::from_utf8(e.name().as_ref())
					.map_err(|e| ApiError::Other(format!("UTF-8 error: {}", e)))?
					.to_string();

				// Handle Image tags with attributes
				if name == "Image" && current_path.contains(&"Images".to_string()) {
					// Extract the url attribute directly and add to images_buffer
					for attr_result in e.attributes() {
						if let Ok(attr) = attr_result {
							if attr.key.as_ref() == b"url" {
								if let Ok(url) = std::str::from_utf8(&attr.value) {
									images_buffer.push(url.to_string());
								}
							}
						}
					}
				}
				// Handle other empty elements (fallback)
				else {
					let text = std::str::from_utf8(e.as_ref())
						.map_err(|e| ApiError::Other(format!("UTF-8 error: {}", e)))?
						.to_string();

					if in_ad {
						current_values.push_str(&text);
					}
				}
			}
			Ok(Event::End(e)) => {
				let name = std::str::from_utf8(e.name().as_ref())
					.map_err(|e| ApiError::Other(format!("UTF-8 error: {}", e)))?
					.to_string();

				if let Some(ad) = &mut current_ad {
					// Special handling for Delivery - store as comma-separated options
					if name == "Delivery" && !delivery_buffer.is_empty() {
						ad.fields
							.insert("Delivery".to_string(), delivery_buffer.join(","));
						delivery_buffer.clear();
					}
					// Special handling for Option elements inside Delivery
					else if name == "Option" && current_path.contains(&"Delivery".to_string()) {
						if !current_values.trim().is_empty() {
							delivery_buffer.push(current_values.trim().to_string());
						}
					} else if name == "Images" {
						// Store image URLs when closing Images tag
						if !images_buffer.is_empty() {
							ad.fields
								.insert("Images".to_string(), images_buffer.join(","));
							images_buffer.clear();
						}
					} else if name == "Image" && current_path.contains(&"Images".to_string()) {
						// For non-empty Image tags, add their text content to images_buffer
						if !current_values.trim().is_empty() {
							images_buffer.push(current_values.trim().to_string());
						}
					}
					// Store other field values if not empty
					else if !current_values.trim().is_empty() && current_path.len() > 1 {
						let field_name = current_path.last().unwrap().clone();
						// Skip storing individual Image and Option elements as they're handled specially
						if field_name != "Image" && field_name != "Option" {
							ad.fields
								.insert(field_name, current_values.trim().to_string());
						}
					}

					// Special handling for Id field
					if name == "Id" {
						if let Some(id_value) = ad.fields.get("Id") {
							ad.id = id_value.clone();
						}
					}
				}

				// If this is the end of an Ad element, add it to the list
				if name == "Ad" {
					in_ad = false;
					if let Some(ad) = current_ad.take() {
						if !ad.id.is_empty() {
							ads.push(ad);
						}
					}
				}

				// println!("End element: {}, current path: {:?}", name, current_path);
				current_path.pop();
				current_values.clear();
			}
			Ok(Event::Eof) => break,
			Err(e) => return Err(ApiError::Other(format!("XML parse error: {}", e))),
			_ => (),
		}

		// Clear the buffer to prevent re-processing the same event
		buf.clear();
	}

	println!("Finished parsing {} ads", ads.len());
	Ok(ads)
}