#![allow(dead_code)]

use log::{debug, error, info};
use std::net::SocketAddr;

use nostr_sdk::client::Error as NostrError;
use nostr_sdk::prelude::{EventId, Metadata, Url};
use nostr_sdk::{Client, Result};

use super::config::NostrConfig;

/// Nostr connection instance.
#[derive(Clone, Debug)]
pub struct NostrInstance {
    pub client: Client,
    pub config: NostrConfig,
}

impl NostrInstance {
    pub async fn new(config: NostrConfig) -> Self {
        let client = Client::new(&config.keys);

        for relay in config.relays.clone() {
            client.add_relay(relay.target, relay.proxy).await.unwrap();
        }

        client.connect().await;

        Self {
            client: client,
            config,
        }
    }

    pub async fn send_message(&self, message: &str) {
        let response = &self.client.publish_text_note(message, &[]).await;

        match response {
            Ok(event_id) => {
                info!("Message sent successfully. Event Id : {:?}", event_id)
            }
            Err(e) => {
                error!("Error on messsaging : {:?}", e);
            }
        }
    }

    // Broadcasts profile metadata (NIP-01) to relays using a NostrConfig
    pub async fn update_profile(&self, _config: &NostrConfig) -> Result<EventId> {
        let mut metadata = Metadata::new();

        if self.config.clone().get_display_name().is_some() {
            // metadata.name(self.config.display_name.clone().unwrap());
            metadata = metadata.display_name(self.config.clone().get_display_name().unwrap());
            metadata = metadata.name(self.config.clone().get_name().unwrap());
        };

        if self.config.clone().get_description().is_some() {
            metadata = metadata.about(self.config.clone().get_description().unwrap());
        };

        if self.config.clone().get_picture().is_some() {
            metadata = metadata
                .picture(Url::parse(self.config.clone().get_picture().unwrap().as_str()).unwrap());
        };

        if self.config.clone().get_banner().is_some() {
            metadata = metadata
                .banner(Url::parse(self.config.clone().get_banner().unwrap().as_str()).unwrap());
        };

        if self.config.clone().get_nip05().is_some() {
            metadata = metadata.nip05(self.config.clone().get_nip05().unwrap());
        };

        if self.config.lud16.is_some() {
            metadata = metadata.lud16(self.config.lud16.clone().unwrap());
        };

        debug!("{:?}", metadata);

        // Broadcast metadata (NIP-01) to relays
        let profile_result = self.get_client().set_metadata(metadata).await.unwrap();

        Ok(profile_result)
    }

    // Add a relay in the current client instance
    pub async fn add_relay(self, url: &str, proxy: Option<SocketAddr>) -> Result<(), NostrError> {
        self.client.add_relay(url, proxy).await
    }
    // Remove a relay in the current client instance
    pub async fn remove_relay(self, url: &str) -> Result<(), NostrError> {
        self.client.remove_relay(url).await
    }

    // Broadcasts message (NIP-02) to nostr relays
    pub async fn publish(self, _message: String) -> Result<()> {
        //  self.client.send_client_msg(message).await;
        Ok(())
    }

    // Get current client instance
    pub fn get_client(&self) -> &Client {
        return &self.client;
    }
}

mod tests {}
