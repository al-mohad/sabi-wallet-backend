use anyhow::{Context, Result};
use secrecy::ExposeSecret;
use sqlx::{
    postgres::{PgPoolOptions, PgPool},
    Pool,
};
use std::time::Duration;
use tracing::info;

use crate::config::Config;

pub type AnyPool = PgPool; // Use PostgreSQL pool directly

pub async fn init_db_pool(config: &Config) -> Result<AnyPool> {
    let database_url = config.database_url.expose_secret();

    let pool = PgPoolOptions::new()
        .max_connections(50)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(30))
        .connect(database_url)
        .await
        .context(format!("Failed to connect to PostgreSQL database at {}", database_url))?;
    
    info!("PostgreSQL database pool created successfully.");
    Ok(pool)
}

pub fn init_redis_client(config: &Config) -> Result<redis::Client> {
    let redis_url = config.redis_url.expose_secret();
    let client = redis::Client::open(redis_url.clone())
        .context(format!("Failed to connect to Redis at {}", redis_url))?;

    info!("Redis client created successfully.");
    Ok(client)
}
