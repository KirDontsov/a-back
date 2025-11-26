use crate::controllers::websocket::WebSocketConnections;
use futures::StreamExt;
use lapin::{
	options::{BasicConsumeOptions, QueueDeclareOptions},
	types::FieldTable,
	Channel,
};
use serde_json::Value;
use std::sync::Arc;

pub struct RabbitMQConsumer;

impl RabbitMQConsumer {
	pub async fn start_consumer(
		rabbitmq_channel: Channel,
		websocket_connections: Arc<WebSocketConnections>,
	) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

		// Create a unique queue for this consumer instance
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

		// Bind the queue to the exchange with a pattern to receive progress updates
		// This will receive all progress messages regardless of user_id
		rabbitmq_channel
			.queue_bind(
				queue.name().as_str(),
				"avito_exchange",
				"progress.*", // Binding pattern to receive all progress messages
				lapin::options::QueueBindOptions::default(),
				FieldTable::default(),
			)
			.await?;

		println!(
			"Declared queue: {} and bound to exchange with pattern: progress.*",
			queue.name()
		);

		// Start consuming messages
		let consumer = rabbitmq_channel
			.basic_consume(
				queue.name().as_str(),
				"crawler_progress_consumer",
				BasicConsumeOptions::default(),
				FieldTable::default(),
			)
			.await?;

		println!("Started consuming from queue: {}", queue.name());

		// Process messages
		let websocket_connections_clone = websocket_connections.clone();
		let mut consumer_stream = consumer;

		while let Some(delivery_result) = consumer_stream.next().await {
			match delivery_result {
				Ok(delivery) => {
					// Process the message
					let message_data = String::from_utf8_lossy(&delivery.data).to_string();
					println!("Received message: {}", message_data);

					// Parse the message as JSON to extract user_id and request_id if present
					match serde_json::from_str::<Value>(&message_data) {
						Ok(json_value) => {
							// Check if the message contains request_id for targeted delivery
							if let Some(request_id) = extract_request_id_from_message(&json_value) {
								// Send the message to specific request's WebSocket connections
								let msg_str = json_value.to_string();
								let connections = websocket_connections_clone.clone();

								tokio::spawn(async move {
									connections
										.broadcast_message_to_request(&request_id, &msg_str)
										.await;
								});
							// Check if the message contains user_id for targeted delivery
							} else if let Some(user_id) = extract_user_id_from_message(&json_value)
							{
								// Send the message to specific user's WebSocket connections
								let msg_str = json_value.to_string();
								let connections = websocket_connections_clone.clone();

								tokio::spawn(async move {
									connections
										.broadcast_message_to_user(&user_id, &msg_str)
										.await;
								});
							} else {
								// Send to all WebSocket connections if no request_id or user_id found
								let msg_str = json_value.to_string();
								let connections = websocket_connections_clone.clone();

								tokio::spawn(async move {
									connections.broadcast_message(&msg_str).await;
								});
							}
						}
						Err(e) => {
							eprintln!("Failed to parse message as JSON: {}", e);
							// Send as string if JSON parsing fails
							let connections = websocket_connections_clone.clone();
							tokio::spawn(async move {
								connections.broadcast_message(&message_data).await;
							});
						}
					}

					// Acknowledge the message
					delivery
						.ack(lapin::options::BasicAckOptions::default())
						.await?;
				}
				Err(e) => {
					eprintln!("Error receiving message: {}", e);
				}
			}
		}

		Ok(())
	}
}

// Helper function to extract user_id from message
fn extract_user_id_from_message(json_value: &Value) -> Option<String> {
	// First try to get user_id directly from the root object
	if let Some(obj) = json_value.as_object() {
		// Try direct user_id field
		if let Some(user_id_val) = obj.get("user_id") {
			if let Some(user_id_str) = user_id_val.as_str() {
				return Some(user_id_str.to_string());
			}
		}

		// Try user_id in a nested request_data object (for CrawlerTask messages)
		if let Some(request_data) = obj.get("request_data").and_then(|v| v.as_object()) {
			if let Some(user_id_val) = request_data.get("user_id") {
				if let Some(user_id_str) = user_id_val.as_str() {
					return Some(user_id_str.to_string());
				}
			}
		}
	}
	None
}

// Helper function to extract request_id from message
fn extract_request_id_from_message(json_value: &Value) -> Option<String> {
	// First try to get request_id directly from the root object
	if let Some(obj) = json_value.as_object() {
		// Try direct request_id field
		if let Some(request_id_val) = obj.get("request_id") {
			if let Some(request_id_str) = request_id_val.as_str() {
				return Some(request_id_str.to_string());
			}
		}
	}
	None
}
