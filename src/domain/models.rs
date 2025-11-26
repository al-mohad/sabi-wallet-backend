use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::types::{Kobo, Sats};

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub phone_number: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Wallet {
    pub id: Uuid,
    pub user_id: Uuid,
    pub nostr_npub: String,
    pub breez_wallet_id: String,
    pub balance_sats: Sats, // Using Sats custom type
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct EncryptedNostrNsec {
    pub id: Uuid,
    pub wallet_id: Uuid,
    pub encrypted_nsec: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub wallet_id: Uuid,
    pub tx_type: String, // Enum might be better here: FiatDeposit, BtcWithdrawal, etc.
    pub amount_sats: Sats, // Using Sats custom type
    pub fee_sats: Sats,
    pub status: String, // Enum might be better: Pending, Completed, Failed
    pub description: Option<String>,
    pub external_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AdminUser {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct FiatOnrampWebhook {
    pub id: Uuid,
    pub provider: String, // e.g., 'paystack', 'gtbank'
    pub event_id: String,
    pub payload: serde_json::Value,
    pub processed: bool,
    pub created_at: DateTime<Utc>,
}
