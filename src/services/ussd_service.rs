use anyhow::Result;
use redis::{AsyncCommands, Client as RedisClient};
use std::time::Duration;
use tracing::{error, info};
use uuid::Uuid;

use crate::{database::AnyPool, error::AppError, utils::phone_number::NigerianPhoneNumber};

const USSD_SESSION_TTL_SECONDS: usize = 180; // 3 minutes as per requirement

// USSD menu states
enum UssdState {
    Start,
    MainMenu,
    CheckBalance,
    SendBitcoin,
    ReceiveBitcoin,
    ConfirmSendBitcoin,
}

impl UssdState {
    fn as_str(&self) -> &'static str {
        match self {
            UssdState::Start => "START",
            UssdState::MainMenu => "MAIN_MENU",
            UssdState::CheckBalance => "CHECK_BALANCE",
            UssdState::SendBitcoin => "SEND_BITCOIN",
            UssdState::ReceiveBitcoin => "RECEIVE_BITCOIN",
            UssdState::ConfirmSendBitcoin => "CONFIRM_SEND_BITCOIN",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s {
            "START" => Some(UssdState::Start),
            "MAIN_MENU" => Some(UssdState::MainMenu),
            "CHECK_BALANCE" => Some(UssdState::CheckBalance),
            "SEND_BITCOIN" => Some(UssdState::SendBitcoin),
            "RECEIVE_BITCOIN" => Some(UssdState::ReceiveBitcoin),
            "CONFIRM_SEND_BITCOIN" => Some(UssdState::ConfirmSendBitcoin),
            _ => None,
        }
    }
}

/// Handles incoming USSD requests, managing session state in Redis.
pub async fn handle_ussd_request(
    redis_client: RedisClient,
    db_pool: AnyPool, // Required for user/wallet lookup
    session_id: &str,
    phone_number: &str,
    text: &str,
) -> Result<String, AppError> {
    let mut con = redis_client.get_async_connection().await?;
    let session_key = format!("ussd:session:{}", session_id);
    let normalized_phone_number = NigerianPhoneNumber::new(phone_number)
        .map_err(|e| AppError::BadRequest(format!("Invalid phone number: {}", e)))?
        .to_string();

    let current_state_str: Option<String> = con.get(&session_key).await?;
    let current_state = current_state_str
        .and_then(|s| UssdState::from_str(&s))
        .unwrap_or(UssdState::Start);

    let response = match current_state {
        UssdState::Start => handle_start_state(&mut con, &session_key, text).await?,
        UssdState::MainMenu => {
            handle_main_menu_state(&mut con, &session_key, text, &normalized_phone_number, db_pool).await?
        }
        UssdState::CheckBalance => handle_check_balance_state(&mut con, &session_key, &normalized_phone_number, db_pool).await?,
        UssdState::SendBitcoin => handle_send_bitcoin_state(&mut con, &session_key, text).await?,
        UssdState::ReceiveBitcoin => handle_receive_bitcoin_state(&mut con, &session_key, &normalized_phone_number, db_pool).await?,
        UssdState::ConfirmSendBitcoin => {
            handle_confirm_send_bitcoin_state(&mut con, &session_key, text, &normalized_phone_number, db_pool).await?
        }
    };

    // Set session expiry
    let _: () = con.expire(&session_key, USSD_SESSION_TTL_SECONDS).await?;

    Ok(response)
}

async fn handle_start_state(
    con: &mut redis::Connection,
    session_key: &str,
    input: &str,
) -> Result<String, AppError> {
    let response = if input.is_empty() {
        // First request in session
        "CON Welcome to Sabi Wallet. Choose an option:\n1. Main Menu".to_string()
    } else if input == "1" {
        "CON Welcome to Sabi Wallet. Choose an option:\n1. Check Balance\n2. Send Bitcoin\n3. Receive Bitcoin".to_string()
    } else {
        "END Invalid input. Please try again.".to_string()
    };
    let _: () = con.set(session_key, UssdState::MainMenu.as_str()).await?;
    Ok(response)
}

async fn handle_main_menu_state(
    con: &mut redis::Connection,
    session_key: &str,
    input: &str,
    phone_number: &str,
    db_pool: AnyPool,
) -> Result<String, AppError> {
    match input {
        "1" => {
            let response = ussd_check_balance(db_pool, phone_number).await?;
            let _: () = con.set(session_key, UssdState::CheckBalance.as_str()).await?;
            Ok(format!("END {}", response))
        }
        "2" => {
            let _: () = con.set(session_key, UssdState::SendBitcoin.as_str()).await?;
            Ok("CON Enter amount in Sats and recipient's address (e.g., 1000 bc1...)".to_string())
        }
        "3" => {
            let response = ussd_receive_bitcoin(db_pool, phone_number).await?;
            let _: () = con.set(session_key, UssdState::ReceiveBitcoin.as_str()).await?;
            Ok(format!("END {}", response))
        }
        _ => {
            let _: () = con.set(session_key, UssdState::MainMenu.as_str()).await?; // Stay in main menu
            Ok("CON Invalid option. Choose:\n1. Check Balance\n2. Send Bitcoin\n3. Receive Bitcoin".to_string())
        }
    }
}

async fn handle_check_balance_state(
    _con: &mut redis::Connection,
    _session_key: &str,
    phone_number: &str,
    db_pool: AnyPool,
) -> Result<String, AppError> {
    let balance = ussd_check_balance(db_pool, phone_number).await?;
    Ok(format!("END Your balance is: {}", balance))
}

async fn handle_send_bitcoin_state(
    con: &mut redis::Connection,
    session_key: &str,
    input: &str,
) -> Result<String, AppError> {
    // Expected format: "amount address" e.g., "1000 bc1..."
    let parts: Vec<&str> = input.splitn(2, ' ').collect();
    if parts.len() != 2 {
        let _: () = con.set(session_key, UssdState::SendBitcoin.as_str()).await?;
        return Ok("CON Invalid format. Enter amount and address (e.g., 1000 bc1...)".to_string());
    }

    let amount_str = parts[0];
    let address = parts[1];

    let amount: i64 = amount_str
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid amount. Must be a number.".to_string()))?;

    // Store details for confirmation
    let _: () = con.hset(session_key, "amount", amount).await?;
    let _: () = con.hset(session_key, "address", address).await?;
    let _: () = con.set(session_key, UssdState::ConfirmSendBitcoin.as_str()).await?;

    Ok(format!(
        "CON Confirm send {} Sats to {}. Reply '1' to confirm, '0' to cancel.",
        amount, address
    ))
}

async fn handle_receive_bitcoin_state(
    _con: &mut redis::Connection,
    _session_key: &str,
    phone_number: &str,
    db_pool: AnyPool,
) -> Result<String, AppError> {
    let address = ussd_receive_bitcoin(db_pool, phone_number).await?;
    Ok(format!("END Your Bitcoin address is: {}", address))
}

async fn handle_confirm_send_bitcoin_state(
    con: &mut redis::Connection,
    session_key: &str,
    input: &str,
    phone_number: &str,
    db_pool: AnyPool,
) -> Result<String, AppError> {
    match input {
        "1" => {
            let amount_str: String = con.hget(session_key, "amount").await?;
            let address: String = con.hget(session_key, "address").await?;
            let amount: i64 = amount_str.parse().map_err(|_| {
                AppError::Internal("Failed to parse amount from session".to_string())
            })?;

            // Perform the actual send Bitcoin operation
            let result = ussd_send_bitcoin(db_pool, phone_number, amount, &address).await;

            // Clear session data after use
            let _: () = con.del(session_key).await?;

            match result {
                Ok(_) => Ok("END Bitcoin sent successfully!".to_string()),
                Err(e) => {
                    error!("Error sending Bitcoin via USSD: {:?}", e);
                    Ok(format!("END Failed to send Bitcoin: {}", e))
                }
            }
        }
        "0" => {
            let _: () = con.del(session_key).await?; // Clear session data
            Ok("END Bitcoin send cancelled.".to_string())
        }
        _ => {
            // Stay in confirmation state
            Ok("CON Invalid input. Reply '1' to confirm, '0' to cancel.".to_string())
        }
    }
}

// --- USSD Core Logic Functions (would interact with wallet/Breez SDK) ---

async fn ussd_check_balance(db_pool: AnyPool, phone_number: &str) -> Result<String, AppError> {
    // Find user and their wallet
    let user_id = sqlx::query_scalar!(
        "SELECT id FROM users WHERE phone_number = $1",
        phone_number
    )
    .fetch_optional(&db_pool)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let balance_sats: i64 = sqlx::query_scalar!(
        "SELECT balance_sats FROM wallets WHERE user_id = $1",
        user_id
    )
    .fetch_one(&db_pool)
    .await?;

    Ok(format!("{} Sats", balance_sats))
}

async fn ussd_send_bitcoin(
    db_pool: AnyPool,
    phone_number: &str,
    amount_sats: i64,
    address: &str,
) -> Result<(), AppError> {
    info!(
        "USSD: User {} attempting to send {} Sats to {}",
        phone_number, amount_sats, address
    );

    // TODO: Implement actual Bitcoin send logic using Breez SDK
    // - Check if user has sufficient balance (atomic operation)
    // - Create a new transaction record (pending)
    // - Call Breez SDK to initiate payment
    // - Update transaction status based on Breez SDK response (or webhook)

    // Placeholder for Breez SDK call
    // breez_sdk::send_payment(amount_sats, address)?;
    tokio::time::sleep(Duration::from_secs(2)).await; // Simulate network call

    // For now, assume success and update balance
    let user_id = sqlx::query_scalar!(
        "SELECT id FROM users WHERE phone_number = $1",
        phone_number
    )
    .fetch_optional(&db_pool)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // This should be done atomically with a proper transaction and balance check
    sqlx::query!(
        "UPDATE wallets SET balance_sats = balance_sats - $1 WHERE user_id = $2 AND balance_sats >= $1",
        amount_sats,
        user_id
    )
    .execute(&db_pool)
    .await?
    .rows_affected()
    .eq(&1)
    .then_some(())
    .ok_or_else(|| AppError::BadRequest("Insufficient balance or wallet not found".to_string()))?;


    info!("Simulated send of {} Sats to {} for user {}", amount_sats, address, phone_number);
    Ok(())
}

async fn ussd_receive_bitcoin(db_pool: AnyPool, phone_number: &str) -> Result<String, AppError> {
    info!("USSD: User {} requesting Bitcoin address", phone_number);

    // Find user and their wallet
    let user_id = sqlx::query_scalar!(
        "SELECT id FROM users WHERE phone_number = $1",
        phone_number
    )
    .fetch_optional(&db_pool)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let wallet_id: Uuid = sqlx::query_scalar!(
        "SELECT id FROM wallets WHERE user_id = $1",
        user_id
    )
    .fetch_one(&db_pool)
    .await?;

    // TODO: Generate new receive address using Breez SDK, or return an existing one
    // For now, return a dummy address for the wallet.
    // breez_sdk::receive_payment(wallet_id)?;
    tokio::time::sleep(Duration::from_secs(1)).await; // Simulate network call
    Ok(format!("bc1qdummyaddressforwallet{}", &wallet_id.to_string()[..8]))
}
