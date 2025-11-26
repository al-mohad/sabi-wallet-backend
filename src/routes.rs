use axum::{routing::post, Router};
use std::sync::Arc;

use crate::{
    api::{admin, recovery, ussd, webhooks},
    app_state::AppState,
};

pub fn api_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .nest("/webhook", webhook_routes(app_state.clone()))
        .nest("/ussd", ussd_routes(app_state.clone()))
        .nest("/recovery", recovery_routes(app_state.clone()))
        .nest("/admin", admin_routes(app_state.clone()))
        .route("/rates", axum::routing::get(webhooks::get_rates_handler)) // Assuming get_rates_handler is in webhooks for now
}

fn webhook_routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/breez", post(webhooks::breez_webhook_handler))
        .route("/paystack", post(webhooks::paystack_webhook_handler))
        .with_state(app_state)
}

fn ussd_routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(ussd::ussd_callback_handler))
        .with_state(app_state)
}

fn recovery_routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/request", post(recovery::request_recovery_handler))
        .route("/submit", post(recovery::submit_share_handler))
        .with_state(app_state)
}

fn admin_routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/login", post(admin::login_handler))
        .route("/trades", axum::routing::get(admin::get_trades_handler))
        .route("/manual-release", post(admin::manual_release_handler))
        .with_state(app_state)
}
