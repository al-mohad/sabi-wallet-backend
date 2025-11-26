use anyhow::{anyhow, Result};
use secrecy::ExposeSecret;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    database::AnyPool,
    domain::models::{EncryptedNostrNsec, Wallet},
    error::AppError,
    nostr::client::{NostrClient, NostrWalletConnectInfo},
};
use nostr_sdk::nostr::key::FromSkStr;
use nostr_sdk::nostr::nips::nip04::decrypt;
use nostr_sdk::nostr::prelude::{Nip04, SecretKey};
use nostr_sdk::nostr::{PublicKey, To){
    pub trait T {
        type Item;
    }
}


/// Initiates the social recovery process by sending encrypted Shamir shares
/// to designated helpers via Nostr DMs.
/// The `wallet_id` is the ID of the wallet that needs recovery.
/// `helper_npubs` are the Nostr public keys of the designated helpers.
pub async fn initiate_recovery_request(
    app_state: Arc<AppState>,
    wallet_id_str: &str,
    helper_npubs: &[String],
) -> Result<(), AppError> {
    info!("Initiating recovery request for wallet: {}", wallet_id_str);

    let wallet_id = Uuid::parse_str(wallet_id_str)
        .map_err(|e| AppError::BadRequest(format!("Invalid wallet ID: {}", e)))?;

    // 1. Retrieve the encrypted nsec from the database for the given wallet.
    let encrypted_nsec_record: EncryptedNostrNsec = sqlx::query_as!(
        EncryptedNostrNsec,
        "SELECT id, wallet_id, encrypted_nsec, created_at FROM encrypted_nostr_nsecs WHERE wallet_id = $1",
        wallet_id
    )
    .fetch_one(&app_state.db_pool)
    .await
    .map_err(|e| AppError::NotFound(format!("Encrypted nsec not found for wallet {}: {}", wallet_id, e)))?;

    // TODO: Decrypt the nsec using a key derived from a master secret or KMS.
    // For now, let's assume `encrypted_nsec_record.encrypted_nsec` is the actual nsec for simplicity,
    // but in production, this MUST be a proper decryption.
    let nsec_secret_key = SecretKey::from_sk_str(&encrypted_nsec_record.encrypted_nsec)
        .map_err(|e| AppError::Internal(format!("Failed to parse nsec for recovery: {}", e)))?;
    let recovery_coordinator_keys = nostr_sdk::Keys::new(nsec_secret_key);

    // 2. Split the nsec into Shamir shares (e.g., 3-of-5).
    // This is a placeholder for `shamir` crate usage.
    let nsec_bytes = recovery_coordinator_keys.secret_key().unwrap().as_bytes();
    let shares = shamir::split_secret(nsec_bytes, 3, 5) // Example: 3 required shares out of 5 total
        .map_err(|e| AppError::Internal(format!("Failed to split secret into shares: {}", e)))?;

    // 3. For each helper, encrypt a share with their public key and send via Nostr DM.
    let nostr_client = NostrClient::new(&app_state.config.nostr_relays)?;

    for (i, npub_str) in helper_npubs.iter().enumerate() {
        let helper_pubkey = PublicKey::from_bech32(npub_str)
            .map_err(|e| AppError::BadRequest(format!("Invalid helper npub '{}': {}", npub_str, e)))?;

        // Encrypt the share using NIP-04 (shared secret encryption)
        let encrypted_share = nostr_sdk::nostr::nips::nip04::encrypt(
            recovery_coordinator_keys.secret_key().unwrap(),
            &helper_pubkey,
            &hex::encode(&shares[i]), // Send the i-th share
        )
        .map_err(|e| AppError::Internal(format!("Failed to encrypt share for helper {}: {}", npub_str, e)))?;

        // Send the encrypted share as a NIP-04 DM
        nostr_client.send_dm(
            &recovery_coordinator_keys, // Pass the Keys object
            helper_pubkey,
            &encrypted_share,
        ).await?;

        info!("Sent share {} to helper {}", i + 1, npub_str);
    }

    Ok(())
}

/// Receives an encrypted share from a helper and stores it temporarily in Redis.
/// When enough shares are collected, it attempts to reconstruct the original nsec.
pub async fn submit_recovery_share(
    app_state: Arc<AppState>,
    wallet_id_str: &str,
    encrypted_share: &str,
    helper_pubkey_str: &str,
) -> Result<(), AppError> {
    info!(
        "Received share for wallet: {} from helper: {}",
        wallet_id_str, helper_pubkey_str
    );

    let wallet_id = Uuid::parse_str(wallet_id_str)
        .map_err(|e| AppError::BadRequest(format!("Invalid wallet ID: {}", e)))?;
    let helper_pubkey = PublicKey::from_bech32(helper_pubkey_str)
        .map_err(|e| AppError::BadRequest(format!("Invalid helper pubkey: {}", e)))?;

    // 1. Retrieve the recovery coordinator's nsec (from config/KMS) to decrypt the share.
    let coordinator_nsec_secret = app_state.config.sabi_nostr_nsec.expose_secret();
    let coordinator_keys = nostr_sdk::Keys::new(SecretKey::from_sk_str(coordinator_nsec_secret)
        .map_err(|e| AppError::Internal(format!("Failed to parse coordinator nsec: {}", e)))?);

    // 2. Decrypt the share using NIP-04.
    let decrypted_share_hex = nostr_sdk::nostr::nips::nip04::decrypt(
        coordinator_keys.secret_key().unwrap(),
        &helper_pubkey,
        encrypted_share,
    )
    .map_err(|e| AppError::BadRequest(format!("Failed to decrypt share: {}", e)))?;

    let decrypted_share = hex::decode(&decrypted_share_hex)
        .map_err(|e| AppError::Internal(format!("Failed to hex-decode decrypted share: {}", e)))?;

    // 3. Store the decrypted share temporarily in Redis (or a secure in-memory store)
    // Key: `recovery:wallet_id:shares`, Value: Hash map of `helper_pubkey` -> `share_data`
    let redis_key = format!("recovery:{}:shares", wallet_id);
    let mut con = app_state.redis_client.get_async_connection().await?;
    let _: () = con.hset(&redis_key, helper_pubkey_str, decrypted_share_hex).await?;
    info!("Stored share for wallet {} from helper {}", wallet_id, helper_pubkey_str);

    // 4. Check if enough shares are collected.
    let collected_shares: Vec<String> = con.hvals(&redis_key).await?;

    if collected_shares.len() >= 3 { // Assuming 3-of-5 scheme
        info!(
            "Enough shares ({}) collected for wallet {}. Attempting reconstruction.",
            collected_shares.len(),
            wallet_id
        );

        let shares_bytes: Result<Vec<Vec<u8>>> = collected_shares
            .into_iter()
            .map(|s| hex::decode(s).map_err(|e| anyhow!("Failed to decode hex share: {}", e)))
            .collect();

        let shares_bytes = shares_bytes.map_err(|e| AppError::Internal(e.to_string()))?;

        let reconstructed_secret = shamir::reconstruct_secret(&shares_bytes)
            .map_err(|e| AppError::Internal(format!("Failed to reconstruct secret: {}", e)))?;

        let reconstructed_nsec = SecretKey::from_slice(&reconstructed_secret)
            .map_err(|e| AppError::Internal(format!("Failed to parse reconstructed nsec: {}", e)))?
            .to_secret_key(); // Convert back to nsec string

        info!("Successfully reconstructed nsec for wallet {}! nsec: {}", wallet_id, reconstructed_nsec.display_secret().to_string());

        // TODO: Store the reconstructed nsec securely or use it immediately to restore wallet access.
        // This reconstructed nsec should NOT be persisted to the database directly unless encrypted with a new key.
        // It's typically used for a one-time operation.

        // Clear shares from Redis after reconstruction
        let _: () = con.del(&redis_key).await?;
    } else {
        info!(
            "Collected {} shares for wallet {}. Need {} more.",
            collected_shares.len(),
            3 - collected_shares.len()
        );
    }

    Ok(())
}
