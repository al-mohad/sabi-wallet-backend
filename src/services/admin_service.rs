use anyhow::{Context, Result};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use secrecy::{ExposeSecret, SecretString};
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    config::Config,
    database::AnyPool,
    domain::{
        models::{AdminUser, Transaction},
        types::Sats,
    },
    error::AppError,
    bitcoin::breez::BreezService,
};

const JWT_EXPIRATION_SECONDS: usize = 3600; // 1 hour

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Claims {
    sub: String, // Subject (admin user ID)
    exp: usize,  // Expiration time
    iat: usize,  // Issued at
}

/// Hashes a password using Argon2.
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?
        .to_string();
    Ok(password_hash)
}

/// Verifies a password against a hash.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(format!("Failed to parse password hash: {}", e)))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Authenticates an admin user and returns a JWT token if successful.
pub async fn authenticate_admin_user(
    db_pool: AnyPool,
    config: Config,
    username: &str,
    password: &str,
) -> Result<String, AppError> {
    // Check if default admin user exists and create if not
    let mut admin_user: Option<AdminUser> = sqlx::query_as!(
        AdminUser,
        "SELECT id, username, password_hash, is_active, created_at, updated_at FROM admin_users WHERE username = $1",
        username
    )
    .fetch_optional(&db_pool)
    .await?;

    if admin_user.is_none() && username == "admin" {
        info!("Default admin user not found, creating one.");
        let default_password_hash = hash_password(config.default_admin_password.expose_secret())?;

        let new_admin_id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO admin_users (id, username, password_hash, is_active, created_at, updated_at) VALUES ($1, $2, $3, TRUE, NOW(), NOW())",
            new_admin_id,
            "admin",
            default_password_hash
        )
        .execute(&db_pool)
        .await?;

        admin_user = Some(AdminUser {
            id: new_admin_id,
            username: "admin".to_string(),
            password_hash: default_password_hash,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });
        info!("Default admin user 'admin' created with default password.");
    }

    let admin = admin_user
        .ok_or_else(|| AppError::Unauthorized("Invalid username or password".to_string()))?;

    if !verify_password(password, &admin.password_hash)? {
        return Err(AppError::Unauthorized("Invalid username or password".to_string()));
    }

    if !admin.is_active {
        return Err(AppError::Forbidden("Admin account is inactive".to_string()));
    }

    // Generate JWT token
    let now = Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: admin.id.to_string(),
        exp: now + JWT_EXPIRATION_SECONDS,
        iat: now,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.app_secret_key.expose_secret().as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to generate JWT token: {}", e)))?;

    Ok(token)
}

/// Fetches all transactions for admin review.
pub async fn fetch_all_transactions(db_pool: AnyPool) -> Result<Vec<Transaction>, AppError> {
    let transactions = sqlx::query_as!(
        Transaction,
        r#"SELECT id, wallet_id, tx_type as "tx_type!", amount_sats as "amount_sats!", fee_sats as "fee_sats!", status as "status!", description, external_id, created_at, updated_at FROM transactions ORDER BY created_at DESC"#
    )
    .fetch_all(&db_pool)
    .await?;

    Ok(transactions)
}

/// Manually releases funds for a given transaction.
/// This would typically involve directly interacting with Breez SDK.
pub async fn manual_release_funds(
    app_state: Arc<AppState>,
    transaction_id_str: &str,
    amount_sats: Sats,
    recipient_nostr_pubkey: &str, // This might be a Bitcoin address or other identifier in reality
    notes: Option<&str>,
) -> Result<(), AppError> {
    let transaction_id = Uuid::parse_str(transaction_id_str)
        .map_err(|e| AppError::BadRequest(format!("Invalid transaction ID: {}", e)))?;

    info!(
        "Admin manual release: Tx ID {}, Amount {} Sats, Recipient {}",
        transaction_id, amount_sats.0, recipient_nostr_pubkey
    );

    // TODO: Verify transaction status in DB, ensure it's pending manual release
    let mut tx = app_state.db_pool.begin().await?;

    let existing_transaction: Option<Transaction> = sqlx::query_as!(
        Transaction,
        r#"SELECT id, wallet_id, tx_type as "tx_type!", amount_sats as "amount_sats!", fee_sats as "fee_sats!", status as "status!", description, external_id, created_at, updated_at FROM transactions WHERE id = $1"#,
        transaction_id
    )
    .fetch_optional(&mut *tx)
    .await?;

    let mut transaction = existing_transaction
        .ok_or_else(|| AppError::NotFound(format!("Transaction {} not found", transaction_id)))?;

    if transaction.status != "pending" && transaction.status != "admin_hold" {
        return Err(AppError::BadRequest(format!(
            "Transaction {} is not in a state for manual release (status: {})",
            transaction_id, transaction.status
        )));
    }

    // Initialize BreezService. This would typically be a singleton in AppState for performance.
    let breez_service = BreezService::new(
        app_state.config.breez_api_key.expose_secret(),
        app_state.config.breez_mnemonic.expose_secret(),
    ).await?;

    // Perform the actual BTC send using Breez SDK
    // For manual release, the recipient_nostr_pubkey might represent an invoice, an address, or another form.
    // Assuming for now it's a simple Bitcoin address for the purpose of this scaffold.
    let payment_info = breez_service.send_payment(amount_sats, recipient_nostr_pubkey).await?;

    info!(
        "Breez SDK payment initiated for manual release. Payment ID: {}",
        payment_info.payment_hash
    );

    // Update transaction status to 'completed' and record external_id
    transaction.status = "completed".to_string();
    transaction.external_id = Some(payment_info.payment_hash);
    transaction.description = notes.map(|s| s.to_string()); // Update notes if provided

    sqlx::query!(
        "UPDATE transactions SET status = $1, external_id = $2, description = $3, updated_at = NOW() WHERE id = $4",
        transaction.status,
        transaction.external_id,
        transaction.description,
        transaction.id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}
