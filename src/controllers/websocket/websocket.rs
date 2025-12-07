use actix_web::{web, HttpRequest, Responder};
use actix_ws::{handle, Message};
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use url::form_urlencoded;
use uuid;

// Define a struct to hold WebSocket connections
#[derive(Clone)]
pub struct WebSocketConnections {
	connections: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<String>>>>,
	user_connections: Arc<RwLock<HashMap<String, Vec<String>>>>, // Maps user_id to connection IDs
	request_connections: Arc<RwLock<HashMap<String, Vec<String>>>>, // Maps request_id to connection IDs
}

impl WebSocketConnections {
	pub fn new() -> Self {
		Self {
			connections: Arc::new(RwLock::new(HashMap::new())),
			user_connections: Arc::new(RwLock::new(HashMap::new())),
			request_connections: Arc::new(RwLock::new(HashMap::new())),
		}
	}

	pub async fn add_connection(
		&self,
		id: String,
		user_id: String,
		sender: mpsc::UnboundedSender<String>,
	) {
		let mut connections = self.connections.write().await;
		connections.insert(id.clone(), sender);

		// Also register this connection with the user
		let mut user_connections = self.user_connections.write().await;
		user_connections
			.entry(user_id)
			.or_insert_with(Vec::new)
			.push(id.clone());
	}

	pub async fn add_request_connection(&self, id: String, request_id: String) {
		// Register this connection with the request_id
		let mut request_connections = self.request_connections.write().await;
		request_connections
			.entry(request_id)
			.or_insert_with(Vec::new)
			.push(id);
	}

	pub async fn remove_connection(&self, id: &str) {
		let mut connections = self.connections.write().await;
		connections.remove(id);

		// Remove from user connections as well
		let mut user_connections = self.user_connections.write().await;
		for (_, user_connection_ids) in user_connections.iter_mut() {
			user_connection_ids.retain(|conn_id| conn_id != id);
		}

		// Remove from request connections as well
		let mut request_connections = self.request_connections.write().await;
		for (_, request_connection_ids) in request_connections.iter_mut() {
			request_connection_ids.retain(|conn_id| conn_id != id);
		}
	}

	pub async fn broadcast_message(&self, message: &str) {
		let connections = self.connections.read().await;
		let connection_ids: Vec<String> = connections.keys().cloned().collect();
		drop(connections); // Release the read lock before sending

		println!(
			"Broadcasting message to {} connections",
			connection_ids.len()
		);

		for conn_id in &connection_ids {
			let connections = self.connections.read().await;
			if let Some(sender) = connections.get(conn_id) {
				if sender.send(message.to_string()).is_err() {
					// Channel is closed, remove the connection
					drop(connections); // Release the read lock before acquiring write lock
					println!(
						"Failed to send message to connection {}, removing connection",
						conn_id
					);
					self.remove_connection(conn_id).await;
				}
			}
		}
	}

	pub async fn broadcast_message_to_user(&self, user_id: &str, message: &str) {
		let user_connections = self.user_connections.read().await;
		if let Some(connection_ids) = user_connections.get(user_id) {
			let connection_ids: Vec<String> = connection_ids.clone();
			drop(user_connections); // Release the read lock before sending

			println!(
				"Broadcasting message to user {} with {} connections",
				user_id,
				connection_ids.len()
			);

			for conn_id in &connection_ids {
				let connections = self.connections.read().await;
				if let Some(sender) = connections.get(conn_id) {
					if sender.send(message.to_string()).is_err() {
						// Channel is closed, remove the connection
						drop(connections); // Release the read lock before acquiring write lock
						println!("Failed to send message to connection {} for user {}, removing connection", conn_id, user_id);
						self.remove_connection(conn_id).await;
					}
				}
			}
		} else {
			println!("No connections found for user {}", user_id);
		}
	}

	pub async fn has_request_connections(&self, request_id: &str) -> bool {
		let request_connections = self.request_connections.read().await;
		request_connections
			.get(request_id)
			.map_or(false, |conns| !conns.is_empty())
	}

	pub async fn broadcast_message_to_request(&self, request_id: &str, message: &str) {
		let request_connections = self.request_connections.read().await;
		if let Some(connection_ids) = request_connections.get(request_id) {
			let connection_ids: Vec<String> = connection_ids.clone();
			drop(request_connections); // Release the read lock before sending

			println!(
				"Broadcasting message to request {} with {} connections",
				request_id,
				connection_ids.len()
			);

			for conn_id in &connection_ids {
				let connections = self.connections.read().await;
				if let Some(sender) = connections.get(conn_id) {
					if sender.send(message.to_string()).is_err() {
						// Channel is closed, remove the connection
						drop(connections); // Release the read lock before acquiring write lock
						println!("Failed to send message to connection {} for request {}, removing connection", conn_id, request_id);
						self.remove_connection(conn_id).await;
					}
				}
			}
		} else {
			println!("No connections found for request {}", request_id);
		}
	}
}

// WebSocket handler function
pub async fn websocket_handler(
	req: HttpRequest,
	body: web::Payload,
	connections: web::Data<WebSocketConnections>,
) -> actix_web::Result<impl Responder> {
	println!(
		"WebSocket connection attempt from: {}",
		req.connection_info().peer_addr().unwrap_or("unknown")
	);

	// Extract user_id from query parameters
	let user_id = extract_user_id_from_request(&req).await.unwrap_or_else(|| {
		// Generate a placeholder user_id if not found
		println!("No user_id found in request, using placeholder");
		uuid::Uuid::nil().to_string()
	});

	// Extract request_id from query parameters
	let request_id = extract_request_id_from_request(&req).await;

	println!(
		"WebSocket connection for user_id: {}, request_id: {:?}",
		user_id, request_id
	);

	// Generate a unique ID for this connection
	let id = uuid::Uuid::new_v4().to_string();
	println!("Generated connection ID: {}", id);

	// Create a channel for sending messages to this connection
	let (tx, mut rx) = mpsc::unbounded_channel::<String>();

	// Add the connection to the global connections map with user_id
	connections
		.add_connection(id.clone(), user_id.clone(), tx)
		.await;
	println!("Added connection to WebSocket connections map");

	// If request_id is provided, register this connection with the request_id
	if let Some(req_id) = request_id {
		connections
			.add_request_connection(id.clone(), req_id.clone())
			.await;
		println!("Registered connection with request_id: {}", req_id);
	}

	// Create the WebSocket context
	let (response, mut session, mut msg_stream) = handle(&req, body)?;
	println!("WebSocket session established successfully");

	// Clone connections for use in the spawned task
	let connections_clone = connections.clone();
	let id_clone = id.clone();

	// Process messages in a spawned task
	actix_web::rt::spawn(async move {
		println!(
			"Started WebSocket message processing loop for connection: {}",
			id_clone
		);
		loop {
			tokio::select! {
				// Handle incoming messages from the WebSocket
				msg_result = msg_stream.next() => {
					match msg_result {
						Some(Ok(Message::Ping(bytes))) => {
							println!("Received ping from connection: {}", id_clone);
							if session.pong(&bytes).await.is_err() {
								println!("Failed to send pong to connection: {}", id_clone);
								break;
							}
						}
						Some(Ok(Message::Text(msg))) => {
							println!("Got text from connection {}: {msg}", id_clone);
						}
						Some(Ok(_)) => {
							// Other message types - continue processing
							continue;
						}
						Some(Err(e)) => {
							// Connection error - break the loop
							println!("Connection error for {}: {:?}", id_clone, e);
							break;
						}
						None => {
							// Connection closed - break the loop
							println!("Connection {} closed by client", id_clone);
							break;
						}
					}
				}
				// Handle outgoing messages to the WebSocket
				msg = rx.recv() => {
					match msg {
						Some(text) => {
							println!("Sending message to connection {}: {}", id_clone, text);
							if session.text(text).await.is_err() {
								println!("Failed to send message to connection: {}", id_clone);
								break;
							}
						}
						None => {
							// Channel closed - break the loop
							println!("Channel closed for connection: {}", id_clone);
							break;
						}
					}
				}
			}
		}

		// Clean up the connection when the loop exits
		println!("Cleaning up connection: {}", id_clone);
		connections_clone.remove_connection(&id_clone).await;
		let _ = session.close(None).await;
		println!("Connection {} closed and cleaned up", id_clone);
	});

	Ok(response)
}

// Helper function to extract user_id from request
async fn extract_user_id_from_request(req: &HttpRequest) -> Option<String> {
	if let Some(query) = req.uri().query() {
		let params: std::collections::HashMap<String, String> =
			form_urlencoded::parse(query.as_bytes())
				.into_owned()
				.collect();
		return params.get("user_id").cloned();
	}
	None
}

// Helper function to extract request_id from request
async fn extract_request_id_from_request(req: &HttpRequest) -> Option<String> {
	if let Some(query) = req.uri().query() {
		let params: std::collections::HashMap<String, String> =
			form_urlencoded::parse(query.as_bytes())
				.into_owned()
				.collect();
		return params.get("request_id").cloned();
	}
	None
}
