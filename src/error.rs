use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Anyhow Error: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("Database Error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Migration Error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Redis Error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Reqwest Error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Serde JSON Error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Validation Error: {0}")]
    Validation(#[from] validator::ValidationErrors),

    // TODO: Nostr SDK error type changed between versions
    // Using String for now until API stabilizes, then should use proper nostr_sdk error type
    #[error("Nostr SDK Error: {0}")]
    NostrClient(String),

    #[error("Authentication Error: {0}")]
    Auth(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not Found: {0}")]
    NotFound(String),

    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal Server Error: {0}")]
    Internal(String),

    #[error("Specific Sabi Wallet Error: {0}")]
    SabiWallet(String),
    // Add more specific error types as needed
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Sqlx(e) => {
                error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
            }
            AppError::Migration(e) => {
                error!("Migration error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database migration error".to_string())
            }
            AppError::Redis(e) => {
                error!("Redis error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Redis error".to_string())
            }
            AppError::Reqwest(e) => {
                error!("HTTP client error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "External service error".to_string())
            }
            AppError::SerdeJson(e) => {
                error!("JSON serialization error: {:?}", e);
                (StatusCode::BAD_REQUEST, "Invalid JSON payload".to_string())
            }
            AppError::Validation(e) => {
                (StatusCode::UNPROCESSABLE_ENTITY, format!("Validation error: {}", e))
            }
            AppError::Nostr(e) => {
                error!("Nostr error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Nostr related error".to_string())
            }
            AppError::Auth(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            AppError::Internal(msg) => {
                error!("Internal Server Error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
            AppError::Anyhow(e) => {
                error!("Unhandled Anyhow error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "An unexpected error occurred".to_string())
            }
            AppError::SabiWallet(msg) => (StatusCode::BAD_REQUEST, msg), // Or a more specific status code depending on context
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
