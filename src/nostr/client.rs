use anyhow::{Context, Result};
use async_trait::async_trait;
use nostr_sdk::nostr::key::{Keys, PublicKey, SecretKey};
use nostr_sdk::nostr::nips::nip04::decrypt;
use nostr_sdk::nostr::{Event, EventId, Filter, Kind, Tag};
use nostr_sdk::{Client, RelayPoolNotification};
use std::time::Duration;
use tracing::{error, info, warn};

use crate::error::AppError;

/// Information required for Nostr Wallet Connect (NWC).
#[derive(Debug, Clone)]
pub struct NostrWalletConnectInfo {
    pub uri: String,
    pub secret: String,
    pub relay_url: String,
}

/// A wrapper around `nostr-sdk` client for Sabi Wallet's specific needs.
pub struct NostrClient {
    client: Client,
    relays: Vec<String>,
}

impl NostrClient {
    /// Creates a new NostrClient and connects to the specified relays.
    pub fn new(relays: &[String]) -> Result<Self, AppError> {
        let keys = Keys::generate(); // Client operates anonymously unless specific keys are passed to functions
        let client = Client::new(&keys);

        // Connect to all relays
        for relay in relays {
            client
                .add_relay(relay)
                .context(format!("Failed to add relay: {}", relay))?;
        }
        tokio::spawn(async move {
            if let Err(e) = client.connect().await {
                error!("Failed to connect to Nostr relays: {}", e);
            }
        });

        info!("Nostr client initialized with relays: {:?}", relays);

        Ok(Self {
            client,
            relays: relays.to_vec(),
        })
    }

    /// Sends a generic Nostr event.
    pub async fn send_event(&self, event: Event) -> Result<EventId, AppError> {
        self.client
            .send_event(event)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to send Nostr event: {}", e)))
    }

    /// Sends a NIP-04 encrypted direct message.
    pub async fn send_dm(
        &self,
        sender_keys: &Keys, // Changed from PublicKey to &Keys
        receiver_pubkey: PublicKey,
        message: &str,
    ) -> Result<EventId, AppError> {
        let event = Event::new_encrypted_direct_msg(
            sender_keys, // Use sender_keys here
            receiver_pubkey,
            message,
            None,
        )
        .map_err(|e| AppError::Internal(format!("Failed to create NIP-04 DM event: {}", e)))?;

        self.send_event(event).await
    }

    /// Fetches events matching the given filters.
    pub async fn get_events_of(&self, filters: Vec<Filter>) -> Result<Vec<Event>, AppError> {
        self.client
            .get_events_of(filters, Some(Duration::from_secs(10)))
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get events from relays: {}", e)))
    }

    /// Gets the URL of the first configured relay. Useful for NWC URI construction.
    pub fn get_first_relay_url(&self) -> &str {
        self.relays.first().map(|s| s.as_str()).unwrap_or_default()
    }

    /// Listens for new events matching a filter. This might be used by the Nostr Relay.
    #[allow(dead_code)] // Will be used by the nostr relay module
    pub async fn listen_for_events(&self, filters: Vec<Filter>) -> Result<(), AppError> {
        info!("Listening for events on relays with filters: {:?}", filters);
        self.client.subscribe(filters).await;

        let mut notifications = self.client.notifications();
        while let Ok(notification) = notifications.recv().await {
            match notification {
                RelayPoolNotification::Event { event, .. } => {
                    info!("Received new event: {:?}", event);
                    // Process event here
                }
                RelayPoolNotification::Message { .. } => {}
                RelayPoolNotification::Shutdown => {
                    info!("Relay pool shut down.");
                    break;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Decrypts a NIP-04 direct message.
    pub fn decrypt_dm(
        sender_pubkey: PublicKey,
        receiver_secret_key: SecretKey,
        encrypted_content: &str,
    ) -> Result<String, AppError> {
        decrypt(&receiver_secret_key, &sender_pubkey, encrypted_content)
            .map_err(|e| AppError::Internal(format!("Failed to decrypt DM: {}", e)))
    }
}
