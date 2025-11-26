// use crate::errors::ApiError;
// use crate::models::{AvitoPriceUpdate, AvitoTokenResponse};
// use backoff::{ExponentialBackoff, Operation};
// use parking_lot::Mutex;
// use reqwest::{Client, StatusCode};
// use serde::Serialize;
// use std::sync::Arc;
// use std::time::{Duration, Instant};
// use tracing::{error, info, warn};

// #[derive(Debug, Clone)]
// pub struct AvitoClient {
//     client: Client,
//     config: AvitoClientConfig,
//     token_cache: Arc<Mutex<TokenCache>>,
//     rate_limiter: governor::RateLimiter<String, governor::state::InMemoryState, governor::clock::DefaultClock, governor::middleware::NoOpMiddleware>,
// }

// #[derive(Debug, Clone)]
// pub struct AvitoClientConfig {
//     pub base_url: String,
//     pub client_id: String,
//     pub client_secret: String,
//     pub max_qps: u32,
//     pub max_retries: u32,
// }

// #[derive(Debug)]
// struct TokenCache {
//     token: String,
//     expires_at: Instant,
// }

// impl AvitoClient {
//     pub fn new(config: AvitoClientConfig) -> Self {
//         let rate_limiter = governor::RateLimiter::keyed(
//             governor::Quota::per_second(config.max_qps).allow_burst(1)
//         );

//         Self {
//             client: Client::new(),
//             config,
//             token_cache: Arc::new(Mutex::new(TokenCache {
//                 token: String::new(),
//                 expires_at: Instant::now(),
//             })),
//             rate_limiter,
//         }
//     }

//     async fn get_token(&self) -> Result<String, ApiError> {
//         let mut cache = self.token_cache.lock();
//         if cache.expires_at > Instant::now() {
//             return Ok(cache.token.clone());
//         }

//         let credentials = format!("{}:{}", self.config.client_id, self.config.client_secret);
//         let encoded = base64::encode(credentials);

//         let response = self.client
//             .post(&format!("{}/token", self.config.base_url))
//             .form(&[("grant_type", "client_credentials")])
//             .header("Authorization", format!("Basic {}", encoded))
//             .send()
//             .await?;

//         if !response.status().is_success() {
//             let error_msg = response.text().await.unwrap_or_default();
//             return Err(ApiError::AvitoAuthError(error_msg));
//         }

//         let token_response: AvitoTokenResponse = response.json().await?;

//         cache.token = token_response.access_token;
//         cache.expires_at = Instant::now() + Duration::from_secs(token_response.expires_in);

//         info!("Refreshed Avito API token");
//         Ok(cache.token.clone())
//     }

    // pub async fn update_price(&self, item_id: &str, price: AvitoPriceUpdate) -> Result<(), ApiError> {
    //     let token = self.get_token().await?;

    //     let operation = || async {
    //         self.rate_limiter.until_key_ready(item_id.to_string()).await;

    //         let url = format!("{}/core/v1/items/{}/price", self.config.base_url, item_id);
    //         let response = self.client
    //             .patch(&url)
    //             .bearer_auth(&token)
    //             .json(&price)
    //             .send()
    //             .await
    //             .map_err(|e| backoff::Error::Permanent(e.into()))?;

    //         match response.status() {
    //             StatusCode::OK => Ok(()),
    //             StatusCode::TOO_MANY_REQUESTS => {
    //                 warn!("Rate limited by Avito API for item {}", item_id);
    //                 Err(backoff::Error::Transient(ApiError::AvitoRateLimited))
    //             }
    //             status if status.is_server_error() => {
    //                 warn!("Avito server error: {}", status);
    //                 Err(backoff::Error::Transient(ApiError::AvitoServerError))
    //             }
    //             _ => {
    //                 let error_msg = response.text().await.unwrap_or_default();
    //                 Err(backoff::Error::Permanent(ApiError::AvitoClientError(error_msg)))
    //             }
    //         }
    //     };

    //     let backoff = ExponentialBackoff {
    //         max_elapsed_time: Some(Duration::from_secs(30)),
    //         ..ExponentialBackoff::default()
    //     };

    //     operation
    //         .retry_with_backoff(&backoff)
    //         .await
    //         .map_err(|e| match e {
    //             backoff::Error::Permanent(inner) => inner,
    //             backoff::Error::Transient(inner) => inner,
    //         })
    // }
}
