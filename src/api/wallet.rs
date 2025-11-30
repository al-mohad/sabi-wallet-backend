use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    error::AppError,
    services::wallet_service::{WalletInfo, WalletService},
};

/// Request to create a Lightning wallet
#[derive(Debug, Deserialize)]
pub struct CreateWalletRequest {
    pub user_id: String,
    pub phone_number: String,
}

/// Response from wallet creation/info endpoint
#[derive(Debug, Serialize)]
pub struct WalletResponse {
    pub success: bool,
    pub data: Option<WalletInfo>,
    pub error: Option<String>,
}

/// Handler to create a new Lightning wallet
///
/// POST /wallet/create
///
/// Creates a Lightning wallet for a user using Breez SDK. Returns wallet
/// connection details including node ID and address.
pub async fn create_wallet_handler(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<CreateWalletRequest>,
) -> Result<(StatusCode, Json<WalletResponse>), AppError> {
    // Parse user_id from request
    let user_id = Uuid::parse_str(&payload.user_id)
        .map_err(|_| AppError::BadRequest("Invalid user_id format".to_string()))?;

    // Validate phone number format
    if !payload.phone_number.starts_with('+') || payload.phone_number.len() < 10 {
        return Err(AppError::BadRequest(
            "Phone number must start with + and be at least 10 digits".to_string(),
        ));
    }

    // Check if user already has a wallet
    let has_wallet = WalletService::user_has_wallet(&app_state.db_pool, user_id)
        .await?;

    if has_wallet {
        return Err(AppError::Conflict(
            "User already has a Lightning wallet".to_string(),
        ));
    }

    // Create the wallet
    let wallet_info = WalletService::create_lightning_wallet(
        &app_state.db_pool,
        user_id,
        &payload.phone_number,
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(WalletResponse {
            success: true,
            data: Some(wallet_info),
            error: None,
        }),
    ))
}

/// Handler to get wallet info for a user
///
/// GET /wallet/:user_id
///
/// Retrieves existing wallet information and connection details.
pub async fn get_wallet_handler(
    State(app_state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> Result<Json<WalletResponse>, AppError> {
    let user_id = Uuid::parse_str(&user_id)
        .map_err(|_| AppError::BadRequest("Invalid user_id format".to_string()))?;

    let wallet_info = WalletService::get_wallet_info(&app_state.db_pool, user_id).await?;

    Ok(Json(WalletResponse {
        success: true,
        data: Some(wallet_info),
        error: None,
    }))
}
