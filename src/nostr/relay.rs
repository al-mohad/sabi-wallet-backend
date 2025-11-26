use nostr_sdk::{
    nostr::{Filter, Event},
    Client,
};
use std::sync::Arc;
use tracing::{info, error};

use crate::{app_state::AppState, error::AppError, nostr::client::NostrClient};

/// Represents the Sabi Wallet's interaction with the Nostr network as a relay client.
/// This module focuses on the Sabi Wallet backend's role in publishing and subscribing
/// to events on *external* Nostr relays, rather than running its own full relay.
pub struct SabiNostrRelayClient {
    client: NostrClient,
    app_state: Arc<AppState>,
}

impl SabiNostrRelayClient {
    pub fn new(app_state: Arc<AppState>) -> Result<Self, AppError> {
        let client = NostrClient::new(&app_state.config.nostr_relays)?;
        Ok(Self { client, app_state })
    }

    /// Starts listening for specific Nostr events relevant to Sabi Wallet.
    /// This could include DMs for social recovery, NWC requests, etc.
    pub async fn start_listening(&self) -> Result<(), AppError> {
        info!("Sabi Nostr Relay Client starting to listen for events...");

        // Example: Listen for DMs directed to the recovery coordinator's pubkey
        let coordinator_keys = self.app_state.config.sabi_nostr_nsec.expose_secret_ref().clone();
        let coordinator_pubkey = nostr_sdk::nostr::key::SecretKey::from_sk_str(&coordinator_keys)
            .map_err(|e| AppError::Internal(format!("Failed to parse coordinator nsec: {}", e)))?
            .public_key();


        let dm_filter = Filter::new()
            .pubkey(coordinator_pubkey)
            .kind(nostr_sdk::nostr::Kind::EncryptedDirectMessage);

        self.client.listen_for_events(vec![dm_filter]).await?;

        Ok(())
    }

    /// Publishes an event to the configured relays.
    pub async fn publish_event(&self, event: Event) -> Result<Event, AppError> {
        let event_id = self.client.send_event(event.clone()).await?;
        info!("Published event {} to relays.", event_id);
        Ok(event)
    }

    /// Placeholder for other relay-like functionalities, e.g.,
    /// - Handling NIP-05 verification
    /// - Processing NIP-46 (Nostr Connect) requests
    /// - Forwarding events to other relays (if acting as a proxy)
    pub async fn process_incoming_event(&self, event: Event) -> Result<(), AppError> {
        info!("Processing incoming event: Kind={}, Author={}", event.kind, event.pubkey);
        // Add specific logic here for different event kinds
        match event.kind {
            nostr_sdk::nostr::Kind::EncryptedDirectMessage => {
                // Handle DMs, potentially for social recovery or other wallet operations
                info!("Received DM event.");
                // TODO: Decrypt and process DM
            }
            _ => {
                info!("Received unhandled event kind: {}", event.kind);
            }
        }
        Ok(())
    }
}
