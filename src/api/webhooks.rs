use axum::{extract::State, http::HeaderMap, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info};
use validator::Validate;

use crate::{
    app_state::AppState,
    domain::types::{Sats, Kobo},
    error::AppError,
    services::fiat_service,
};

#[derive(Debug, Deserialize, Validate)]
pub struct BreezWebhookRequest {
    // Breez SDK webhook payload details
    // Example fields, adjust based on actual Breez webhook structure
    pub payment_hash: String,
    pub bolt11: Option<String>,
    pub amount_msat: u64,
    pub fee_msat: u64,
    pub status: String, // e.g., "PAID", "FAILED"
    // Other fields as necessary
}

#[derive(Debug, Serialize)]
pub struct BreezWebhookResponse {
    pub success: bool,
    pub message: String,
}

/// POST /webhook/breez
/// Receives payment confirmation from Breez SDK.
pub async fn breez_webhook_handler(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<BreezWebhookRequest>,
) -> Result<Json<BreezWebhookResponse>, AppError> {
    info!("Received Breez webhook for payment hash: {}", payload.payment_hash);

    // TODO: Implement replay protection and signature verification on Breez webhooks
    // This is CRITICAL for security. The Breez SDK documentation should provide details.
    if let Some(signature) = headers.get("X-Breez-Signature") {
        debug!("Breez webhook signature: {:?}", signature);
        // Verify signature here
    } else {
        error!("Breez webhook received without signature header.");
        // Depending on policy, this might be an immediate rejection
    }

    payload.validate()?;

    // Delegate processing to a service layer
    fiat_service::process_breez_payment(
        app_state.db_pool.clone(),
        payload.payment_hash,
        Sats(payload.amount_msat as i64 / 1000), // convert msats to sats
        Sats(payload.fee_msat as i64 / 1000),
        payload.status,
    )
    .await?;

    Ok(Json(BreezWebhookResponse {
        success: true,
        message: "Breez webhook processed successfully".to_string(),
    }))
}

#[derive(Debug, Deserialize, Validate)]
pub struct PaystackWebhookRequest {
    // Paystack webhook payload details
    // Example fields, adjust based on actual Paystack webhook structure
    pub event: String, // e.g., "charge.success"
    pub data: PaystackTransactionData,
}

#[derive(Debug, Deserialize, Validate)]
pub struct PaystackTransactionData {
    pub id: u64,
    pub domain: String,
    pub status: String, // e.g., "success"
    pub reference: String,
    pub amount: u64, // Amount in kobo
    pub currency: String, // "NGN"
    pub customer: PaystackCustomer,
    // Add other relevant fields
}

#[derive(Debug, Deserialize)]
pub struct PaystackCustomer {
    pub id: u64,
    pub email: String,
    pub phone: Option<String>,
    // Other customer details
}


#[derive(Debug, Serialize)]
pub struct PaystackWebhookResponse {
    pub success: bool,
    pub message: String,
}

/// POST /webhook/paystack
/// Receives Naira transfer confirmation from Paystack → triggers BTC send.
pub async fn paystack_webhook_handler(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<PaystackWebhookRequest>,
) -> Result<Json<PaystackWebhookResponse>, AppError> {
    info!(
        "Received Paystack webhook for event: {} and reference: {}",
        payload.event, payload.data.reference
    );

    // TODO: Verify Paystack webhook signature using PAYSTACK_SECRET_KEY
    // This is CRITICAL for security.
    // Example: headers.get("x-paystack-signature")
    if let Some(signature) = headers.get("x-paystack-signature") {
        debug!("Paystack webhook signature: {:?}", signature);
        // Implement signature verification using app_state.config.paystack_secret_key.expose_secret()
    } else {
        error!("Paystack webhook received without signature header.");
        // Depending on policy, this might be an immediate rejection
    }

    // TODO: Implement idempotency key check (e.g., using X-Idempotency-Key header or a field in payload)
    // This is CRITICAL to prevent double processing.

    if payload.event == "charge.success" && payload.data.status == "success" {
        // Delegate processing to a service layer
        fiat_service::process_paystack_deposit(
            app_state.db_pool.clone(),
            app_state.redis_client.clone(),
            payload.data.reference,
            Kobo(payload.data.amount as i64),
            payload.data.customer.phone, // Phone number might need canonicalization
            payload.data.customer.email,
        )
        .await?;

        Ok(Json(PaystackWebhookResponse {
            success: true,
            message: "Paystack webhook processed successfully".to_string(),
        }))
    } else {
        info!("Paystack webhook not 'charge.success' or status not 'success'. Ignoring.");
        Ok(Json(PaystackWebhookResponse {
            success: true, // Acknowledge receipt even if not processed
            message: "Webhook received, but not eligible for processing".to_string(),
        }))
    }
}

#[derive(Debug, Serialize)]
pub struct RatesResponse {
    pub naira_to_btc: f64,
    pub last_updated_at: String,
}

/// GET /rates
/// Returns cached Naira -> BTC rate.
pub async fn get_rates_handler(
    State(app_state): State<Arc<AppState>>,
) -> Result<Json<RatesResponse>, AppError> {
    info!("Fetching current BTC/Naira rates.");
    // Delegate to a service layer that fetches from multiple sources with fallback and caching
    let (rate, last_updated) = fiat_service::get_cached_btc_naira_rate(app_state.redis_client.clone()).await?;

    Ok(Json(RatesResponse {
        naira_to_btc: rate,
        last_updated_at: last_updated.to_rfc3339(),
    }))
}
