use crate::database::AnyPool;
use crate::domain::types::Sats;
use crate::error::AppError;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tracing::info;
use uuid::Uuid;

/// Connection details for a Lightning wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConnectionDetails {
    pub wallet_id: String,
    pub user_id: String,
    pub lightning_node_id: String,
    pub node_address: String,
    pub synced: bool,
    pub initialized_at: String,
}

/// Lightning wallet info response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub id: Uuid,
    pub user_id: Uuid,
    pub breez_wallet_id: String,
    pub nostr_npub: String,
    pub balance_sats: i64,
    pub connection_details: WalletConnectionDetails,
    pub created_at: String,
}

pub struct WalletService;

impl WalletService {
    /// Creates a new Lightning wallet for a user
    ///
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `user_id` - The user ID
    /// * `phone_number` - User's phone number
    ///
    /// # Returns
    /// Created wallet info with connection details
    pub async fn create_lightning_wallet(
        pool: &AnyPool,
        user_id: Uuid,
        phone_number: &str,
    ) -> Result<WalletInfo, AppError> {
        info!("Creating Lightning wallet for user: {}", user_id);

        // Generate wallet identifiers
        let wallet_id = Uuid::new_v4();
        let breez_wallet_id = format!("breez_{}", wallet_id.to_string().replace("-", ""));
        let node_id = format!("node_{}", Uuid::new_v4().to_string().replace("-", ""));

        // Generate a mock Nostr public key (npub) - in production, this comes from Nostr account
        let nostr_npub = format!("npub1{}", Uuid::new_v4().to_string().replace("-", "").chars().take(56).collect::<String>());

        // For now, generate a mock node address (in production, this comes from Breez SDK)
        let node_address = format!("lnd_node_{}@127.0.0.1:9735", &node_id[5..15]);

        // Insert wallet into database
        let now = Utc::now();
        let result = sqlx::query(
            r#"
            INSERT INTO wallets (id, user_id, nostr_npub, breez_wallet_id, balance_sats, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, user_id, breez_wallet_id, nostr_npub, balance_sats, created_at
            "#
        )
        .bind(wallet_id)
        .bind(user_id)
        .bind(&nostr_npub)
        .bind(&breez_wallet_id)
        .bind(0i64)  // Initial balance: 0 sats
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            match e {
                sqlx::Error::RowNotFound => AppError::NotFound("User not found".to_string()),
                sqlx::Error::Database(db_err) if db_err.message().contains("duplicate key") => {
                    AppError::Conflict("Wallet already exists for this user".to_string())
                }
                _ => AppError::Sqlx(e),
            }
        })?;

        let created_at: chrono::DateTime<chrono::Utc> = result.get("created_at");

        let wallet_info = WalletInfo {
            id: wallet_id,
            user_id,
            breez_wallet_id: result.get("breez_wallet_id"),
            nostr_npub: result.get("nostr_npub"),
            balance_sats: result.get("balance_sats"),
            connection_details: WalletConnectionDetails {
                wallet_id: wallet_id.to_string(),
                user_id: user_id.to_string(),
                lightning_node_id: node_id,
                node_address,
                synced: false, // Initial state, will be synced after first channel open
                initialized_at: now.to_rfc3339(),
            },
            created_at: created_at.to_rfc3339(),
        };

        info!("Lightning wallet created successfully: {}", wallet_id);
        Ok(wallet_info)
    }

    /// Gets wallet info for a user
    pub async fn get_wallet_info(
        pool: &AnyPool,
        user_id: Uuid,
    ) -> Result<WalletInfo, AppError> {
        info!("Fetching wallet info for user: {}", user_id);

        let wallet_row = sqlx::query(
            r#"
            SELECT id, user_id, breez_wallet_id, nostr_npub, balance_sats, created_at
            FROM wallets
            WHERE user_id = $1
            LIMIT 1
            "#
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        let wallet_row = wallet_row.ok_or_else(|| {
            AppError::NotFound("Wallet not found for this user".to_string())
        })?;

        let wallet_id: Uuid = wallet_row.get("id");
        let breez_wallet_id: String = wallet_row.get("breez_wallet_id");
        let created_at: chrono::DateTime<chrono::Utc> = wallet_row.get("created_at");

        // Generate connection details
        let node_id = format!("node_{}", Uuid::new_v4().to_string().replace("-", ""));
        let node_address = format!("lnd_node_{}@127.0.0.1:9735", &node_id[5..15]);

        Ok(WalletInfo {
            id: wallet_id,
            user_id,
            breez_wallet_id,
            nostr_npub: wallet_row.get("nostr_npub"),
            balance_sats: wallet_row.get("balance_sats"),
            connection_details: WalletConnectionDetails {
                wallet_id: wallet_id.to_string(),
                user_id: user_id.to_string(),
                lightning_node_id: node_id,
                node_address,
                synced: true,
                initialized_at: created_at.to_rfc3339(),
            },
            created_at: created_at.to_rfc3339(),
        })
    }

    /// Checks if a user already has a wallet
    pub async fn user_has_wallet(pool: &AnyPool, user_id: Uuid) -> Result<bool, AppError> {
        let result = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM wallets WHERE user_id = $1)"
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(result)
    }
}
