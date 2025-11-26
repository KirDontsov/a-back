mod api;
mod config;
mod controllers;
mod jwt_auth;
mod models;
mod utils;

use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use actix_web_grants::GrantsMiddleware;
use config::Config;
use dotenv::dotenv;
use lapin::Channel;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::sync::Arc;

use crate::controllers::auth::extract;

pub struct AppState {
	db: Pool<Postgres>,
	rabbitmq_channel: Channel,
	env: Config,
	websocket_connections: web::Data<crate::controllers::websocket::WebSocketConnections>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	if std::env::var_os("RUST_LOG").is_none() {
		std::env::set_var("RUST_LOG", "actix_web=info");
	}
	dotenv().ok();
	env_logger::init();

	let config = Config::init();

	let pool = match PgPoolOptions::new()
		.max_connections(10)
		.connect(&config.database_url)
		.await
	{
		Ok(pool) => {
			println!("✅ Connection to the database is successful!");
			pool
		}
		Err(err) => {
			println!("🔥 Failed to connect to the database: {:?}", err);
			std::process::exit(1);
		}
	};

	// Connect to RabbitMQ
	let conn =
		match lapin::Connection::connect(&config.rabbitmq_url, lapin::ConnectionProperties::default())
			.await
		{
			Ok(pool) => {
				println!("✅ Connection to the RabbitMQ is successful!");
				pool
			}
			Err(err) => {
				println!("🔥 Failed to connect to the RabbitMQ: {:?}", err);
				std::process::exit(1);
			}
		};
	let channel = match conn.create_channel().await {
		Ok(pool) => {
			println!("✅ RabbitMQ Channel established successfuly!");
			pool
		}
		Err(err) => {
			println!("🔥 Failed to connect to the RabbitMQ: {:?}", err);
			std::process::exit(1);
		}
	};
	// Add this before publishing to check channel state
	log::debug!("Channel status: {:?}", channel.status());
	log::debug!("Channel state: {:?}", channel.status().state());

	// Create WebSocket connections manager
	let websocket_connections = crate::controllers::websocket::WebSocketConnections::new();
	let websocket_connections_data = web::Data::new(websocket_connections.clone());

	// Start RabbitMQ consumer
	let rabbitmq_channel_clone = channel.clone();
	let ws_connections_clone = Arc::new(websocket_connections.clone());
	tokio::spawn(async move {
		if let Err(e) = crate::controllers::rabbitmq_consumer::RabbitMQConsumer::start_consumer(
			rabbitmq_channel_clone,
			ws_connections_clone,
		)
		.await
		{
			eprintln!("Error starting RabbitMQ consumer: {}", e);
		}
	});

	println!("✅ Server started successfully on http://localhost:8081/api");

	HttpServer::new(move || {
	let auth = GrantsMiddleware::with_extractor(extract);
		App::new()
			.app_data(web::Data::new(AppState {
				db: pool.clone(),
				rabbitmq_channel: channel.clone(),
				env: config.clone(),
				websocket_connections: websocket_connections_data.clone(),
			}))
			.configure(controllers::config)
			.wrap(Cors::permissive())
			.wrap(Logger::default())
			.wrap(auth)
	})
	.bind(("127.0.0.1", 8081))?
	.run()
	.await
}
