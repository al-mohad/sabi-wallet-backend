use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;
use validator::Validate;

use crate::{app_state::AppState, error::AppError, services::recovery_service};

#[derive(Debug, Deserialize, Validate)]
pub struct RecoveryRequestPayload {
    #[validate(length(min = 1, message = "Wallet ID is required"))]
    pub wallet_id: String, // Uuid as string
    #[validate(length(min = 1, message = "Nostr npub of helper is required"))]
    pub helper_npubs: Vec<String>, // List of Nostr npubs of helpers
}

#[derive(Debug, Serialize)]
pub struct RecoveryRequestResponse {
    pub success: bool,
    pub message: String,
}

/// POST /recovery/request
/// Initiates recovery share requests via Nostr.
pub async fn request_recovery_handler(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<RecoveryRequestPayload>,
) -> Result<Json<RecoveryRequestResponse>, AppError> {
    info!(
        "Received recovery request for wallet ID: {}",
        payload.wallet_id
    );

    payload.validate()?;

    recovery_service::initiate_recovery_request(
        app_state.clone(),
        &payload.wallet_id,
        &payload.helper_npubs,
    )
    .await?;

    Ok(Json(RecoveryRequestResponse {
        success: true,
        message: "Recovery request initiated successfully".to_string(),
    }))
}

#[derive(Debug, Deserialize, Validate)]
pub struct SubmitSharePayload {
    #[validate(length(min = 1, message = "Wallet ID is required"))]
    pub wallet_id: String, // Uuid as string
    #[validate(length(min = 1, message = "Encrypted share is required"))]
    pub encrypted_share: String,
    #[validate(length(min = 1, message = "Nostr pubkey of helper is required"))]
    pub helper_pubkey: String, // Nostr pubkey of the helper who sent the share
}

#[derive(Debug, Serialize)]
pub struct SubmitShareResponse {
    pub success: bool,
    pub message: String,
}

/// POST /recovery/submit
/// Receives encrypted share from a helper.
pub async fn submit_share_handler(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<SubmitSharePayload>,
) -> Result<Json<SubmitShareResponse>, AppError> {
    info!(
        "Received recovery share for wallet ID: {} from helper: {}",
        payload.wallet_id, payload.helper_pubkey
    );

    payload.validate()?;

    recovery_service::submit_recovery_share(
        app_state.clone(),
        &payload.wallet_id,
        &payload.encrypted_share,
        &payload.helper_pubkey,
    )
    .await?;

    Ok(Json(SubmitShareResponse {
        success: true,
        message: "Recovery share submitted successfully".to_string(),
    }))
}
