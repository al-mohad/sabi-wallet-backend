use axum::{extract::State, Form};
use serde::Deserialize;
use std::sync::Arc;
use tracing::info;

use crate::{app_state::AppState, error::AppError, services::ussd_service};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UssdRequest {
    // Africa's Talking specific fields
    pub session_id: String,
    pub service_code: String,
    pub phone_number: String,
    pub text: String, // User's input
}

/// POST /ussd
/// Handles Africa's Talking USSD callbacks.
pub async fn ussd_callback_handler(
    State(app_state): State<Arc<AppState>>,
    Form(payload): Form<UssdRequest>,
) -> Result<String, AppError> {
    info!(
        "USSD Request: Session ID={}, Phone={}, Text='{}'",
        payload.session_id, payload.phone_number, payload.text
    );

    // Delegate to the USSD service to handle the logic and generate the response
    let response_text = ussd_service::handle_ussd_request(
        app_state.redis_client.clone(),
        app_state.db_pool.clone(),
        &payload.session_id,
        &payload.phone_number,
        &payload.text,
    )
    .await?;

    Ok(response_text)
}
