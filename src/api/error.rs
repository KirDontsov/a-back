use actix_web::{body::BoxBody, http::StatusCode, HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// pub type CustomResult<T> = Result<T, CustomError>;

#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum CustomError {
	#[error("Endpoint is not found: {0}")]
	NotFound(String),
	#[error("Internal server error: {0}")]
	InternalError(String),
}

impl ResponseError for CustomError {
	fn status_code(&self) -> StatusCode {
		match self {
			Self::NotFound(_) => StatusCode::NOT_FOUND,
			Self::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
		}
	}

	fn error_response(&self) -> HttpResponse<BoxBody> {
		log::error!("Error: {}", self.to_string());
		HttpResponse::build(self.status_code()).json(self)
	}
}

impl From<sqlx::Error> for CustomError {
	fn from(source: sqlx::Error) -> Self {
		Self::InternalError(source.to_string())
	}
}
