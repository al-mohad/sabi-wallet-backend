use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;
use validator::Validate;

use crate::{
    app_state::AppState,
    domain::{models::Transaction, types::Sats},
    error::AppError,
    services::admin_service,
};

#[derive(Debug, Deserialize, Validate)]
pub struct AdminLoginPayload {
    #[validate(length(min = 1, message = "Username cannot be empty"))]
    pub username: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AdminLoginResponse {
    pub success: bool,
    pub message: String,
    pub token: Option<String>, // JWT token
}

/// POST /admin/login
/// Handles admin login and issues a JWT token upon successful authentication.
pub async fn login_handler(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<AdminLoginPayload>,
) -> Result<Json<AdminLoginResponse>, AppError> {
    info!("Admin login attempt for user: {}", payload.username);

    payload.validate()?;

    let token = admin_service::authenticate_admin_user(
        app_state.db_pool.clone(),
        app_state.config.clone(),
        &payload.username,
        &payload.password,
    )
    .await?;

    // TODO: Implement 2FA via Nostr NIP-46 or TOTP after initial login.
    // This would likely involve an additional step/endpoint or a more complex token.

    Ok(Json(AdminLoginResponse {
        success: true,
        message: "Login successful".to_string(),
        token: Some(token),
    }))
}

#[derive(Debug, Serialize)]
pub struct AdminTradesResponse {
    pub trades: Vec<Transaction>, // Detailed transaction information
}

/// GET /admin/trades
/// Retrieves a list of all trades/transactions for admin review.
pub async fn get_trades_handler(
    State(app_state): State<Arc<AppState>>,
    // TODO: Add authentication layer here (e.g., `axum_extra::extract::PrivateClaim<Claims>`)
    // _auth_guard: AuthGuard, // Placeholder for an authentication guard
) -> Result<Json<AdminTradesResponse>, AppError> {
    info!("Admin requested all trades.");

    let trades = admin_service::fetch_all_transactions(app_state.db_pool.clone()).await?;

    Ok(Json(AdminTradesResponse { trades }))
}

#[derive(Debug, Deserialize, Validate)]
pub struct ManualReleasePayload {
    #[validate(length(min = 1, message = "Transaction ID is required"))]
    pub transaction_id: String, // UUID as string
    pub amount_sats: Sats,
    #[validate(length(min = 1, message = "Recipient Nostr public key is required"))]
    pub recipient_nostr_pubkey: String, // Npub of the recipient
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ManualReleaseResponse {
    pub success: bool,
    pub message: String,
    pub transaction_id: String,
}

/// POST /admin/manual-release
/// Allows an admin to manually release funds for a given transaction.
pub async fn manual_release_handler(
    State(app_state): State<Arc<AppState>>,
    // TODO: Add authentication layer here
    Json(payload): Json<ManualReleasePayload>,
) -> Result<Json<ManualReleaseResponse>, AppError> {
    info!(
        "Admin initiated manual release for transaction ID: {}",
        payload.transaction_id
    );

    payload.validate()?;

    admin_service::manual_release_funds(
        app_state.clone(),
        &payload.transaction_id,
        payload.amount_sats,
        &payload.recipient_nostr_pubkey,
        payload.notes.as_deref(),
    )
    .await?;

    Ok(Json(ManualReleaseResponse {
        success: true,
        message: format!(
            "Funds manually released for transaction ID: {}",
            payload.transaction_id
        ),
        transaction_id: payload.transaction_id,
    }))
}
