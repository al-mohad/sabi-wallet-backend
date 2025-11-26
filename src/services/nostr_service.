use anyhow::Result;
use nostr_sdk::nostr::{Event, EventId, Filter, Keys, Kind, Tag};
use std::sync::Arc;
use tracing::{info, warn};

use crate::{
    app_state::AppState,
    error::AppError,
    nostr::client::{NostrClient, NostrWalletConnectInfo},
};

/// Service for Nostr-related operations, including key management and relay interaction.
pub struct NostrService {
    client: NostrClient,
    // Add other dependencies like database pool if needed for persistence
}

impl NostrService {
    pub fn new(app_state: Arc<AppState>) -> Result<Self, AppError> {
        let client = NostrClient::new(&app_state.config.nostr_relays)?;
        Ok(Self { client })
    }

    /// Publishes an event to the configured Nostr relays.
    pub async fn publish_event(&self, event: Event) -> Result<EventId, AppError> {
        info!("Publishing Nostr event: {}", event.id.to_bech32()?);
        let event_id = self.client.send_event(event).await?;
        info!("Event published: {}", event_id.to_bech32()?);
        Ok(event_id)
    }

    /// Sends a direct message (NIP-04) from `sender_keys` to `receiver_pubkey`.
    pub async fn send_direct_message(
        &self,
        sender_keys: Keys,
        receiver_pubkey: Keys,
        message: &str,
    ) -> Result<EventId, AppError> {
        info!(
            "Sending DM from {} to {}",
            sender_keys.public_key().to_bech32()?,
            receiver_pubkey.public_key().to_bech32()?
        );
        let event_id = self
            .client
            .send_dm(sender_keys, receiver_pubkey.public_key(), message)
            .await?;
        info!("DM sent: {}", event_id.to_bech32()?);
        Ok(event_id)
    }

    /// Fetches events matching a given filter from the relays.
    pub async fn fetch_events(&self, filter: Filter) -> Result<Vec<Event>, AppError> {
        info!("Fetching Nostr events with filter: {:?}", filter);
        let events = self.client.get_events_of(vec![filter]).await?;
        info!("Fetched {} events.", events.len());
        Ok(events)
    }

    /// Generates a new Nostr keypair and returns it.
    pub fn generate_new_keypair() -> Keys {
        Keys::generate()
    }

    /// Derives NWC info for a given wallet (not fully implemented, just a placeholder)
    pub async fn derive_nwc_info(
        &self,
        wallet_id: &Uuid,
        user_pubkey: &PublicKey,
    ) -> Result<NostrWalletConnectInfo, AppError> {
        // In a real scenario, this would involve creating/retrieving a secure NWC secret
        // for the user's wallet, linking it to their pubkey, and constructing the URI.
        warn!("NWC derivation is a placeholder. Implement proper secure NWC setup.");

        let relay_url = self.client.get_first_relay_url().to_string();
        let secret = Keys::generate().secret_key()?.display_secret().to_string(); // Dummy secret
        let nwc_uri = format!("nostrwalletconnect://{}?relay={}&secret={}",
            user_pubkey.to_bech32()?, relay_url, secret);

        Ok(NostrWalletConnectInfo {
            uri: nwc_uri,
            secret,
            relay_url: relay_url.to_string(),
        })
    }
}
