use anyhow::Result;
use axum::{extract::Request, http::StatusCode, routing::get, Router};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    request_id::{MakeRequestId, RequestId, SetRequestIdLayer},
    trace::TraceLayer,
    validate_request::ValidateRequestHeaderLayer,
};
use tracing::{info, Level};
use uuid::Uuid;

mod api;
mod app_state;
mod bitcoin;
mod cli;
mod config;
mod database;
mod domain;
mod error;
mod nostr;
mod routes;
mod services;
mod utils;

use app_state::AppState;
use config::Config;
use database::AnyPool;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    dotenvy::dotenv().ok();
    let config = Config::load()?;

    // Initialize tracing
    setup_tracing(&config);

    info!("Starting Sabi Wallet Backend in {} environment", config.app_env);

    // Initialize Sentry for error tracking
    setup_sentry(&config);

    // Initialize Database
    let db_pool: AnyPool = database::init_db_pool(&config).await?;
    sqlx::migrate!("./migrations").run(&db_pool).await?;
    info!("Database migrations applied successfully.");

    // Initialize Redis
    let redis_client = database::init_redis_client(&config)?;

    // Build shared application state
    let app_state = AppState::new(config.clone(), db_pool, redis_client);

    let app = create_app(app_state)?;

    let addr = SocketAddr::from(([127, 0, 0, 1], config.server_port));
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn create_app(app_state: Arc<AppState>) -> Result<Router> {
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .nest("/api", routes::api_router(app_state.clone()))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(SetRequestIdLayer::x_request_id(make_uuid_request_id()))
                .layer(ValidateRequestHeaderLayer::x_request_id()),
        )
        .layer(cors)
        .with_state(app_state);

    Ok(app)
}

fn setup_tracing(config: &Config) {
    let filter_layer = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .from_env_lossy();

    tracing_subscriber::fmt()
        .with_env_filter(filter_layer)
        .json() // Structured logging
        .init();
}

fn setup_sentry(config: &Config) {
    if !config.sentry_dsn.is_empty() {
        sentry::init((
            config.sentry_dsn.clone(),
            sentry::ClientOptions {
                release: sentry::release_name!(),
                environment: Some(config.app_env.to_string().into()),
                debug: config.app_env == "dev",
                ..Default::default()
            },
        ));
        info!("Sentry initialized for environment: {}", config.app_env);
    } else {
        info!("Sentry DSN not provided, Sentry not initialized.");
    }
}

// Custom request ID generation
#[derive(Clone)]
struct UuidRequestId;

impl MakeRequestId for UuidRequestId {
    fn make_request_id<B>(&mut self, _: &Request<B>) -> Option<RequestId> {
        let request_id = Uuid::new_v4().to_string();
        Some(RequestId::new(request_id.parse().unwrap()))
    }
}

fn make_uuid_request_id() -> UuidRequestId {
    UuidRequestId
}
