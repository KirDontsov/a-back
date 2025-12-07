use crate::{controllers::websocket::WebSocketConnections, jwt_auth::JwtMiddleware, AppState};
use actix_web::{
	post,
	web::{self},
	HttpResponse, Responder,
};
use chrono::{Datelike, NaiveDate, Utc};
use futures::StreamExt;
use lapin::{options::*, types::FieldTable, Channel};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct AiTitleProcessingMessage {
	pub task_id: Uuid,
	pub user_id: Uuid,
	pub title: Option<String>,
	pub category: String,
	pub created_ts: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiTitleProcessingResult {
	pub task_id: Uuid,
	pub user_id: Uuid,
	pub request_id: Uuid,
	pub status: String,
	pub result_data: AiResultData,
	pub error_message: Option<String>,
	pub completed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiResultData {
	pub beautified_title: String,
}

#[derive(Deserialize)]
pub struct AiTitleProcessingRequest {
	pub title: String,
	pub category: String,
}

// Helper function to parse and check if the ad date is not today
fn is_date_not_today(ad_date_str: &str) -> bool {
	// Parse the date string which might be in format "DD.MM.YYYY в HH:MM" or similar
	let today = Utc::now().date_naive();

	// Extract the date part from the string (before " в " if present)
	let date_part = if let Some(pos) = ad_date_str.find(" в ") {
		&ad_date_str[..pos].trim()
	} else {
		ad_date_str
	};

	// Parse the date in DD.MM.YYYY format
	if let Ok(parsed_date) = NaiveDate::parse_from_str(date_part, "%d.%m.%Y") {
		// Compare with today's date
		parsed_date != today
	} else {
		// If parsing fails, return true (consider it as not today)
		true
	}
}

// Create AI title processing task handler
#[post("/ai_title_processing")]
pub async fn create_ai_title_processing_handler(
	body: web::Json<AiTitleProcessingRequest>,
	data: web::Data<AppState>,
	user: JwtMiddleware,
) -> impl Responder {
	let user_id = user.user_id;
	let title = &body.title;
	let category = &body.category;

	// Validate input
	if category.trim().is_empty() {
		return HttpResponse::BadRequest().json(json!({
			"status": "error",
			"message": "Category cannot be empty"
		}));
	}

	// Precondition: Find requests matching the category and get related ads
	// SELECT from postgres all the requests WHERE request LIKE '%{category}%', take the newest one
	let requests_query = "
        SELECT request_id, user_id, request, city, coords, radius, district, created_ts, updated_ts
        FROM avito_requests
        WHERE request LIKE $1
        ORDER BY created_ts DESC
        LIMIT 1
    ";

	let category_pattern = format!("%{}%", category);
	let found_request_result =
		sqlx::query_as::<_, crate::models::avito_requests::AvitoRequest>(&requests_query)
			.bind(&category_pattern)
			.fetch_optional(&data.db)
			.await;

	// Query to find the newest request matching the category
	let title = if let Ok(Some(request)) = found_request_result {
		let request_id = request.request_id;

		// Query to find ads with the specified conditions for the specific request
		let ads_query = "
            SELECT *
            FROM avito_analytics_ads
            WHERE avito_request_id = $1
            AND promotion = ''
            AND ad_date != $2
            ORDER BY position
            LIMIT 1
        ";

		// Get today's date in DD.MM.YYYY format to compare with ad_date
		let today = Utc::now();
		let today_formatted = format!("{:02}.{:02}.{}", today.day(), today.month(), today.year());

		let ad_result = sqlx::query_as::<_, crate::models::avito_requests::AdRecord>(&ads_query)
			.bind(request_id)
			.bind(&today_formatted)
			.fetch_optional(&data.db)
			.await;

		if let Ok(Some(ad)) = ad_result {
			// Check if the ad date is not today (since ad_date is in string format like "24.10.2025 в 15:57")
			if is_date_not_today(&ad.ad_date) {
				// Print the ad (as requested in the task)
				println!(
					"Found ad: ID={}, Title={}",
					ad.avito_ad_id, ad.title
				);

				// Return the title from the ad as the best title
				Some(ad.title)
			} else {
				None
			}
		} else {
			None
		}
	} else {
		None
	};

	// Create message
	let message = AiTitleProcessingMessage {
		task_id: Uuid::new_v4(),
		user_id,
		title,
		category: category.clone(),
		created_ts: chrono::Utc::now(),
	};

	// Publish to RabbitMQ using the channel
	match publish_ai_title_processing(&data.rabbitmq_channel, &message).await {
		Ok(_) => {
			// Just return immediately, the response will come via WebSocket
			HttpResponse::Ok().json(json!({
				"status": "success",
				"message": "AI title processing task created",
				"data": {
					"task_id": message.task_id,
					"user_id": message.user_id,
					"title": message.title,
					"category": message.category,
				}
			}))
		}
		Err(e) => {
			log::error!("Failed to publish AI title processing message: {}", e);
			HttpResponse::InternalServerError().json(json!({
				"status": "error",
				"message": "Failed to send task to queue",
				"error": e.to_string()
			}))
		}
	}
}

// Message publishing function
async fn publish_ai_title_processing(
	rabbitmq_channel: &lapin::Channel,
	message: &AiTitleProcessingMessage,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
	let message_json = serde_json::to_string(message)?;

	// Declare exchange
	rabbitmq_channel
		.exchange_declare(
			"avito_exchange",
			lapin::ExchangeKind::Topic,
			lapin::options::ExchangeDeclareOptions {
				durable: true,
				..lapin::options::ExchangeDeclareOptions::default()
			},
			lapin::types::FieldTable::default(),
		)
		.await?;

	// Declare the queue for AI processing tasks
	rabbitmq_channel
		.queue_declare(
			"ai_processing_tasks", // Specific queue name for AI processing tasks
			lapin::options::QueueDeclareOptions {
				durable: true,      // Make the queue durable
				auto_delete: false, // Don't auto-delete the queue
				..lapin::options::QueueDeclareOptions::default()
			},
			lapin::types::FieldTable::default(),
		)
		.await?;

	// Publish directly to the queue (not exchange)
	rabbitmq_channel
		.basic_publish(
			"",                    // Default exchange (direct to queue)
			"ai_processing_tasks", // queue name
			lapin::options::BasicPublishOptions::default(),
			message_json.as_bytes(),
			lapin::BasicProperties::default().with_headers({
				let mut headers = lapin::types::FieldTable::default();
				headers.insert(
					"user_id".into(),
					lapin::types::AMQPValue::LongString(message.user_id.to_string().into()),
				);
				headers.insert(
					"task_id".into(),
					lapin::types::AMQPValue::LongString(message.task_id.to_string().into()),
				);
				headers
			}),
		)
		.await?;

	log::info!(
		"Published AI title processing message for user: {} to queue: ai_processing_tasks",
		message.user_id
	);
	Ok(())
}

// Function to listen for AI response and send via WebSocket (currently not used directly in the handler)
// This function can be used if we need to wait for a response synchronously in other scenarios
async fn listen_for_ai_response(
	rabbitmq_channel: Channel,
	websocket_connections: Arc<WebSocketConnections>,
	task_id: Uuid,
	user_id: Uuid,
) -> Result<AiTitleProcessingResult, Box<dyn std::error::Error + Send + Sync>> {
	// Declare the exchange
	rabbitmq_channel
		.exchange_declare(
			"avito_exchange",
			lapin::ExchangeKind::Topic,
			lapin::options::ExchangeDeclareOptions {
				durable: true,
				..lapin::options::ExchangeDeclareOptions::default()
			},
			FieldTable::default(),
		)
		.await?;

	// Create a unique queue for this specific task response
	let queue = rabbitmq_channel
		.queue_declare(
			"", // Let RabbitMQ generate a unique queue name
			QueueDeclareOptions {
				durable: false,
				exclusive: true, // This queue will be deleted when the connection closes
				auto_delete: true,
				..QueueDeclareOptions::default()
			},
			FieldTable::default(),
		)
		.await?;

	// Bind the queue to listen for results for this specific user
	// The AI microservice sends responses with routing key like "result.{user_id}"
	rabbitmq_channel
		.queue_bind(
			queue.name().as_str(),
			"avito_exchange",
			&format!("result.{}", user_id),
			lapin::options::QueueBindOptions::default(),
			FieldTable::default(),
		)
		.await?;

	log::info!(
		"Declared queue: {} and bound to exchange with pattern: result.{}",
		queue.name(),
		user_id
	);

	// Start consuming messages
	let consumer = rabbitmq_channel
		.basic_consume(
			queue.name().as_str(),
			"ai_title_response_consumer",
			BasicConsumeOptions::default(),
			FieldTable::default(),
		)
		.await?;

	log::info!(
		"Started consuming from queue: {} for task {}",
		queue.name(),
		task_id
	);

	let mut consumer_stream = consumer;

	// Set a timeout for waiting for the response (e.g., 30 seconds)
	let timeout_future = tokio::time::sleep(tokio::time::Duration::from_secs(30));
	tokio::pin!(timeout_future);

	loop {
		tokio::select! {
			// Wait for message from RabbitMQ
			delivery_result = consumer_stream.next() => {
				match delivery_result {
					Some(Ok(delivery)) => {
						// Process the message
						let message_data = String::from_utf8_lossy(&delivery.data).to_string();
						log::info!("Received AI response message: {}", message_data);

						// Parse the message as JSON to check if it's the response for our task
						match serde_json::from_str::<AiTitleProcessingResult>(&message_data) {
							Ok(ai_response) => {
								if ai_response.task_id == task_id {
									// Acknowledge the message
									delivery
										.ack(lapin::options::BasicAckOptions::default())
										.await?;

									// Send the response via WebSocket to the client
									let msg_str = serde_json::to_string(&ai_response)?;
									websocket_connections
										.broadcast_message_to_user(&user_id.to_string(), &msg_str)
										.await;

									log::info!("Sent AI response via WebSocket for task: {}", task_id);
									return Ok(ai_response);
								} else {
									// Not our task, continue listening
									delivery
										.ack(lapin::options::BasicAckOptions::default())
										.await?;
									continue;
								}

							}
							Err(e) => {
								log::error!("Failed to parse AI response as expected format: {}", e);
								// Acknowledge the message even if parsing failed
								delivery
									.ack(lapin::options::BasicAckOptions::default())
									.await?;
								continue;
							}
						}
					}
					Some(Err(e)) => {
						log::error!("Error receiving message: {}", e);
						return Err(Box::new(e));
					}
					None => {
						log::warn!("Consumer stream ended unexpectedly");
						return Err("Consumer stream ended unexpectedly".into());
					}
				}
			}
			// Timeout case
			_ = &mut timeout_future => {
				return Err("Timeout waiting for AI response".into());
			}
		}
	}
}
