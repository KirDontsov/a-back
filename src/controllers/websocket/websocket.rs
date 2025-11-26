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
		for (_, sender) in connections.iter() {
			let _ = sender.send(message.to_string());
		}
	}

	pub async fn broadcast_message_to_user(&self, user_id: &str, message: &str) {
		let user_connections = self.user_connections.read().await;
		if let Some(connection_ids) = user_connections.get(user_id) {
			let connections = self.connections.read().await;
			for conn_id in connection_ids {
				if let Some(sender) = connections.get(conn_id) {
					let _ = sender.send(message.to_string());
				}
			}
		}
	}

	pub async fn broadcast_message_to_request(&self, request_id: &str, message: &str) {
		let request_connections = self.request_connections.read().await;
		if let Some(connection_ids) = request_connections.get(request_id) {
			let connections = self.connections.read().await;
			for conn_id in connection_ids {
				if let Some(sender) = connections.get(conn_id) {
					let _ = sender.send(message.to_string());
				}
			}
		}
	}
}

// WebSocket handler function
pub async fn websocket_handler(
	req: HttpRequest,
	body: web::Payload,
	connections: web::Data<WebSocketConnections>,
) -> actix_web::Result<impl Responder> {
	// Extract user_id from query parameters
	let user_id = extract_user_id_from_request(&req).await.unwrap_or_else(|| {
		// Generate a placeholder user_id if not found
		uuid::Uuid::nil().to_string()
	});

	// Extract request_id from query parameters
	let request_id = extract_request_id_from_request(&req).await;

	// Generate a unique ID for this connection
	let id = uuid::Uuid::new_v4().to_string();

	// Create a channel for sending messages to this connection
	let (tx, mut rx) = mpsc::unbounded_channel::<String>();

	// Add the connection to the global connections map with user_id
	connections
		.add_connection(id.clone(), user_id.clone(), tx)
		.await;

	// If request_id is provided, register this connection with the request_id
	if let Some(req_id) = request_id {
		connections.add_request_connection(id.clone(), req_id).await;
	}

	// Create the WebSocket context
	let (response, mut session, mut msg_stream) = handle(&req, body)?;

	// Clone connections for use in the spawned task
	let connections_clone = connections.clone();
	let id_clone = id.clone();

	// Process messages in a spawned task
	actix_web::rt::spawn(async move {
		loop {
			tokio::select! {
				// Handle incoming messages from the WebSocket
				msg_result = msg_stream.next() => {
					match msg_result {
						Some(Ok(Message::Ping(bytes))) => {
							if session.pong(&bytes).await.is_err() {
								break;
							}
						}
						Some(Ok(Message::Text(msg))) => {
							println!("Got text: {msg}");
						}
						Some(Ok(_)) => {
							// Other message types - continue processing
							continue;
						}
						Some(Err(_)) => {
							// Connection error - break the loop
							break;
						}
						None => {
							// Connection closed - break the loop
							break;
						}
					}
				}
				// Handle outgoing messages to the WebSocket
				msg = rx.recv() => {
					match msg {
						Some(text) => {
							if session.text(text).await.is_err() {
								break;
							}
						}
						None => {
							// Channel closed - break the loop
							break;
						}
					}
				}
			}
		}

		// Clean up the connection when the loop exits
		connections_clone.remove_connection(&id_clone).await;
		let _ = session.close(None).await;
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
