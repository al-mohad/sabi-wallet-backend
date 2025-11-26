use anyhow::{Context, Result};
use secrecy::ExposeSecret;
use sqlx::{
    postgres::{PgPoolOptions},
    sqlite::{SqlitePoolOptions},
    Pool, Any,
};
use std::time::Duration;
use tracing::info;

use crate::config::Config;

pub type AnyPool = Pool<Any>; // Define a type alias for convenience

pub async fn init_db_pool(config: &Config) -> Result<AnyPool> {
    let database_url = config.database_url.expose_secret();

    if database_url.starts_with("sqlite:") {
        let pool = SqlitePoolOptions::new()
            .max_connections(50)
            .acquire_timeout(Duration::from_secs(30))
            .connect(database_url)
            .await
            .context(format!("Failed to connect to SQLite database at {}", database_url))?;
        info!("SQLite database pool created successfully.");
        Ok(pool.into()) // Convert SqlitePool to AnyPool
    } else {
        // Assume PostgreSQL if not SQLite
        let pool = PgPoolOptions::new()
            .max_connections(50)
            .min_connections(5)
            .acquire_timeout(Duration::from_secs(30))
            .connect(database_url)
            .await
            .context(format!("Failed to connect to PostgreSQL database at {}", database_url))?;
        info!("PostgreSQL database pool created successfully.");
        Ok(pool.into()) // Convert PgPool to AnyPool
    }
}

pub fn init_redis_client(config: &Config) -> Result<redis::Client> {
    let redis_url = config.redis_url.expose_secret();
    let client = redis::Client::open(redis_url.clone())
        .context(format!("Failed to connect to Redis at {}", redis_url))?;

    info!("Redis client created successfully.");
    Ok(client)
}
