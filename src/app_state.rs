use std::sync::Arc;

use redis::Client as RedisClient;

use crate::{config::Config, database::AnyPool};

/// Shared application state for Axum handlers.
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db_pool: AnyPool,
    pub redis_client: RedisClient,
    // Other services (e.g., Nostr client, Breez SDK client) will be added here
}

impl AppState {
    pub fn new(config: Config, db_pool: AnyPool, redis_client: RedisClient) -> Arc<Self> {
        Arc::new(Self {
            config,
            db_pool,
            redis_client,
        })
    }
}
