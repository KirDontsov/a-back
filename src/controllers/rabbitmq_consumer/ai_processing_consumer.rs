use crate::controllers::websocket::WebSocketConnections;
use futures::StreamExt;
use lapin::{
	options::{BasicConsumeOptions, QueueDeclareOptions},
	types::FieldTable,
	Channel,
};
use serde_json::Value;
use std::sync::Arc;

pub struct AIProcessingConsumer;

impl AIProcessingConsumer {
	pub async fn start_ai_consumer(
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

		// Bind the queue to the exchange with patterns to receive AI processing responses
		// This will receive both progress updates and final results for AI processing
		rabbitmq_channel
			.queue_bind(
				queue.name().as_str(),
				"avito_exchange",
				"result.*", // Binding pattern to receive all result messages
				lapin::options::QueueBindOptions::default(),
				FieldTable::default(),
			)
			.await?;

		println!(
			"Declared queue: {} and bound to exchange with patterns: result.* and progress.*",
			queue.name()
		);

		// Start consuming messages
		let consumer = rabbitmq_channel
			.basic_consume(
				queue.name().as_str(),
				"ai_result_progress_consumer", // Consumer tag
				BasicConsumeOptions::default(),
				FieldTable::default(),
			)
			.await?;

		println!(
			"Started consuming avito progress messages from queue: {}",
			queue.name()
		);

		// Process messages
		let websocket_connections_clone = websocket_connections.clone();
		let mut consumer_stream = consumer;

		while let Some(delivery_result) = consumer_stream.next().await {
			match delivery_result {
				Ok(delivery) => {
					// Process the message
					let message_data = String::from_utf8_lossy(&delivery.data).to_string();
					println!("Received AI processing message: {}", message_data);

					// Parse the message as JSON to extract user_id and request_id if present
					match serde_json::from_str::<Value>(&message_data) {
						Ok(json_value) => {
							println!("Parsed JSON message: {:?}", json_value);

							// Extract user_id for debugging
							if let Some(user_id) = extract_user_id_from_message(&json_value) {
								println!("Extracted user_id: {}", user_id);
							} else {
								println!("No user_id found in message");
							}

							// Extract request_id for debugging
							if let Some(request_id) = extract_request_id_from_message(&json_value) {
								println!("Extracted request_id: {}", request_id);
							} else {
								println!("No request_id found in message");
							}
						}
						Err(e) => {
							eprintln!("Failed to parse message as JSON: {}", e);
						}
					}

					// Parse the message as JSON to extract user_id and request_id if present
					match serde_json::from_str::<Value>(&message_data) {
						Ok(json_value) => {
							// Check if the message contains request_id for targeted delivery
							if let Some(request_id) = extract_request_id_from_message(&json_value) {
								// Send the message to specific request's WebSocket connections
								let msg_str = json_value.to_string();
								let connections = websocket_connections_clone.clone();

								tokio::spawn(async move {
									println!(
										"Attempting to broadcast message to request: {}",
										request_id
									);
									// Check if there are connections for this request_id
									let has_request_connections =
										connections.has_request_connections(&request_id).await;

									if has_request_connections {
										connections
											.broadcast_message_to_request(&request_id, &msg_str)
											.await;
										println!("Broadcast to request {} completed", request_id);
									} else {
										// No connections for this request_id, try user_id instead
										if let Some(user_id) =
											extract_user_id_from_message(&json_value)
										{
											println!("No connections found for request {}, falling back to user {}", request_id, user_id);
											connections
												.broadcast_message_to_user(&user_id, &msg_str)
												.await;
											println!("Broadcast to user {} completed", user_id);
										} else {
											println!("No user_id found to fall back to, broadcasting to all connections");
											connections.broadcast_message(&msg_str).await;
										}
									}
								});
							// Check if the message contains user_id for targeted delivery
							} else if let Some(user_id) = extract_user_id_from_message(&json_value)
							{
								// Send the message to specific user's WebSocket connections
								let msg_str = json_value.to_string();
								let connections = websocket_connections_clone.clone();

								tokio::spawn(async move {
									println!(
										"Attempting to broadcast message to user: {}",
										user_id
									);
									connections
										.broadcast_message_to_user(&user_id, &msg_str)
										.await;
									println!("Broadcast to user {} completed", user_id);
								});
							} else {
								// Send to all WebSocket connections if no request_id or user_id found
								let msg_str = json_value.to_string();
								let connections = websocket_connections_clone.clone();

								tokio::spawn(async move {
									println!("Broadcasting message to all connections");
									connections.broadcast_message(&msg_str).await;
									println!("Broadcast to all connections completed");
								});
							}
						}
						Err(e) => {
							eprintln!("Failed to parse AI processing message as JSON: {}", e);
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
					eprintln!("Error receiving AI processing message: {}", e);
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
