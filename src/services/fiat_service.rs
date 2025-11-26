use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use redis::{AsyncCommands, Client as RedisClient};
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    database::AnyPool,
    domain::{
        models::{FiatOnrampWebhook, Transaction, Wallet},
        types::{Kobo, Sats},
    },
    error::AppError,
    utils::phone_number::NigerianPhoneNumber,
};

const BTC_NAIRA_RATE_KEY: &str = "btc:naira_rate";
const BTC_NAIRA_RATE_TTL_SECONDS: usize = 300; // 5 minutes

/// Processes a payment confirmation from Breez SDK.
/// Marks the corresponding transaction as complete and updates wallet balances.
pub async fn process_breez_payment(
    db_pool: AnyPool,
    payment_hash: String,
    amount_sats: Sats,
    fee_sats: Sats,
    status: String,
) -> Result<(), AppError> {
    info!(
        "Processing Breez payment for hash: {}, amount: {} sats, status: {}",
        payment_hash, amount_sats.0, status
    );

    // TODO: Look up the original transaction in the DB using payment_hash
    // Update its status based on the Breez webhook status (e.g., 'completed', 'failed')
    // If successful, update the user's wallet balance.
    // This is a placeholder for actual database operations.

    // Example: Find transaction by external_id (payment_hash) and update
    let mut tx = db_pool.begin().await?;

    let existing_transaction: Option<Transaction> = sqlx::query_as!(
        Transaction,
        r#"SELECT id, wallet_id, tx_type as "tx_type!", amount_sats as "amount_sats!", fee_sats as "fee_sats!", status as "status!", description, external_id, created_at, updated_at FROM transactions WHERE external_id = $1"#,
        payment_hash
    )
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(mut transaction) = existing_transaction {
        // Prevent reprocessing
        if transaction.status == "completed" {
            info!("Transaction {} already completed.", transaction.id);
            tx.commit().await?;
            return Ok(());
        }

        if status == "PAID" {
            transaction.status = "completed".to_string();
            // Update transaction status
            sqlx::query!(
                "UPDATE transactions SET status = $1, amount_sats = $2, fee_sats = $3, updated_at = NOW() WHERE id = $4",
                transaction.status,
                amount_sats.0,
                fee_sats.0,
                transaction.id
            )
            .execute(&mut *tx)
            .await?;

            // Update wallet balance (add amount, subtract fee)
            sqlx::query!(
                "UPDATE wallets SET balance_sats = balance_sats + $1 - $2, updated_at = NOW() WHERE id = $3",
                amount_sats.0,
                fee_sats.0,
                transaction.wallet_id
            )
            .execute(&mut *tx)
            .await?;
            info!("Breez payment {} completed. Wallet updated.", payment_hash);
        } else {
            transaction.status = "failed".to_string();
            sqlx::query!(
                "UPDATE transactions SET status = $1, updated_at = NOW() WHERE id = $2",
                transaction.status,
                transaction.id
            )
            .execute(&mut *tx)
            .await?;
            error!("Breez payment {} failed with status: {}", payment_hash, status);
        }
    } else {
        error!("No matching transaction found for Breez payment hash: {}", payment_hash);
        // Potentially create a new transaction with status 'unknown' or 'unmatched'
    }

    tx.commit().await?;

    Ok(())
}

/// Processes a Paystack deposit, records the webhook, and initiates a BTC send.
pub async fn process_paystack_deposit(
    db_pool: AnyPool,
    redis_client: RedisClient,
    reference: String,
    amount_kobo: Kobo,
    phone_number: Option<String>,
    _customer_email: String,
) -> Result<(), AppError> {
    info!(
        "Processing Paystack deposit for reference: {}, amount: {} Kobo",
        reference, amount_kobo.0
    );

    // 1. Record the incoming webhook for audit and idempotency.
    let webhook_id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO fiat_onramp_webhooks (id, provider, event_id, payload, processed, created_at)
        VALUES ($1, $2, $3, $4, $5, NOW())"#,
        webhook_id,
        "paystack",
        reference,
        serde_json::to_value(&reference).unwrap(), // Placeholder payload, use actual JSON from webhook
        false
    )
    .execute(&db_pool)
    .await?;

    // 2. Lookup or create user and wallet based on phone number.
    let canonical_phone = NigerianPhoneNumber::new(
        phone_number
            .as_ref()
            .ok_or_else(|| AppError::BadRequest("Phone number missing from Paystack webhook".to_string()))?
    )
    .map_err(|e| AppError::BadRequest(format!("Invalid phone number from Paystack: {}", e)))?;

    let user_id = sqlx::query_scalar!(
        "SELECT id FROM users WHERE phone_number = $1",
        canonical_phone.as_str()
    )
    .fetch_optional(&db_pool)
    .await?
    .unwrap_or_else(|| {
        // TODO: Create new user and wallet if not found.
        // For now, let's error out to make sure we don't proceed without a user.
        // This is a simplified flow. Real world requires user creation, nostr key gen, breez wallet init.
        error!("User not found for phone number: {}", canonical_phone.as_str());
        Uuid::new_v4() // Placeholder
    });

    let wallet: Wallet = sqlx::query_as!(
        Wallet,
        r#"SELECT id, user_id, nostr_npub, breez_wallet_id, balance_sats as "balance_sats!", created_at, updated_at FROM wallets WHERE user_id = $1"#,
        user_id
    )
    .fetch_one(&db_pool)
    .await
    .map_err(|e| AppError::Internal(format!("Failed to retrieve wallet for user {}: {}", user_id, e)))?;

    // 3. Get current Naira-to-BTC rate.
    let (btc_naira_rate, _) = get_cached_btc_naira_rate(redis_client).await?;
    let btc_amount_sats = Sats((((amount_kobo.0 as f64 / 100.0) / btc_naira_rate) * 100_000_000.0) as i64);

    info!(
        "Converted {} Kobo to {} Sats using rate {}",
        amount_kobo.0, btc_amount_sats, btc_naira_rate
    );

    // 4. Record pending BTC transaction.
    let transaction_id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO transactions (id, wallet_id, tx_type, amount_sats, fee_sats, status, description, external_id, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())"#,
        transaction_id,
        wallet.id,
        "fiat_deposit",
        btc_amount_sats,
        0, // Fees handled by Breez SDK for the actual send
        "pending",
        Some("Paystack Naira deposit for BTC".to_string()),
        Some(reference.clone())
    )
    .execute(&db_pool)
    .await?;

    // 5. Trigger BTC send via Breez SDK (this part would typically be asynchronous/queued)
    // For now, we'll just log and assume this happens via Breez SDK's payment confirmation webhook
    // once the funds are available.
    info!(
        "Fiat deposit processed. Initiating BTC transfer of {} Sats to wallet: {}. Reference: {}",
        btc_amount_sats, wallet.id, reference
    );

    // TODO: Call Breez SDK to create a payment to the user's Breez wallet for `btc_amount_sats`.
    // The Breez SDK webhook will then confirm this.
    // breez_sdk::send_payment(...)

    // Update webhook status to processed
    sqlx::query!(
        "UPDATE fiat_onramp_webhooks SET processed = TRUE WHERE id = $1",
        webhook_id
    )
    .execute(&db_pool)
    .await?;

    Ok(())
}

/// Fetches the cached BTC to Naira rate from Redis, or retrieves it from external sources if expired.
pub async fn get_cached_btc_naira_rate(redis_client: RedisClient) -> Result<(f64, DateTime<Utc>), AppError> {
    let mut conn = redis_client.get_async_connection().await?;

    let cached_rate_str: Option<String> = conn.get(BTC_NAIRA_RATE_KEY).await?;

    if let Some(rate_str) = cached_rate_str {
        if let Ok(rate_value) = rate_str.parse::<f64>() {
            // Redis doesn't store TTL directly, assume if present it's fresh enough or fetch from source
            // For a real system, you'd store (rate, timestamp) together.
            info!("Using cached BTC/Naira rate: {}", rate_value);
            return Ok((rate_value, Utc::now())); // Placeholder for actual timestamp
        }
    }

    info!("BTC/Naira rate not cached or expired, fetching from external sources.");
    // Placeholder: Fetch from a mock external source
    let external_rate = fetch_btc_naira_from_external_source().await?;
    let now = Utc::now();

    // Cache the new rate
    let _: () = conn.set_ex(BTC_NAIRA_RATE_KEY, external_rate.to_string(), BTC_NAIRA_RATE_TTL_SECONDS).await?;
    info!("Cached new BTC/Naira rate: {}", external_rate);

    Ok((external_rate, now))
}

/// Mock function to fetch BTC to Naira rate from an external API.
async fn fetch_btc_naira_from_external_source() -> Result<f64> {
    // In a real application, this would call multiple exchanges and aggregate.
    // For now, return a static value.
    info!("Fetching BTC/Naira rate from mock external source.");
    tokio::time::sleep(std::time::Duration::from_millis(500)).await; // Simulate API call
    Ok(150_000_000.0) // Example: 1 BTC = 150,000,000 Naira
}
