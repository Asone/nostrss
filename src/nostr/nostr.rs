#![allow(dead_code)]

use log::{debug, error, info};
use std::net::SocketAddr;

use nostr_sdk::client::Error as NostrError;
use nostr_sdk::prelude::{EventId, Metadata, Url};
use nostr_sdk::{Client, Keys, Result};

use crate::profiles::config::Profile;

use crate::nostr::relay::Relay;

// Helper trait.
pub trait NostrProfile {
    fn get_keys(&self) -> Keys;
    fn get_display_name(self) -> Option<String>;
    fn get_description(self) -> Option<String>;
    fn get_picture(self) -> Option<String>;
    fn get_banner(self) -> Option<String>;
    fn get_nip05(self) -> Option<String>;
    fn get_lud16(self) -> Option<String>;
    fn get_relays(&self) -> Vec<Relay>;
}
/// Nostr connection instance.
///
/// NostrInstance provides a nostr client instance with a loaded configuration.
/// This allows to use multiple clients instances with multiples profiles and identities.
///
#[derive(Clone, Debug)]
pub struct NostrInstance {
    pub client: Client,
    pub config: Profile,
}

impl NostrInstance {
    pub async fn new(config: Profile) -> Self {
        let keys = &config.get_keys();
        let client = Client::new(keys);

        for relay in &config.get_relays().clone() {
            let target: &String = &relay.target.clone();
            client.add_relay(target, relay.proxy).await.unwrap();
        }

        client.connect().await;

        Self { client, config }
    }

    // Broadcast message to network (NIP-02)
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

    // Broadcasts profile metadata (NIP-01) to relays using a
    pub async fn update_profile(&self) -> Result<EventId> {
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
            let parsed_url = Url::parse(self.config.clone().get_picture().unwrap().as_str());

            if parsed_url.is_ok() {
                metadata = metadata.picture(parsed_url.unwrap());
            }
        };

        if self.config.clone().get_banner().is_some() {
            let parsed_url = Url::parse(self.config.clone().get_banner().unwrap().as_str());

            if parsed_url.is_ok() {
                metadata = metadata.banner(parsed_url.unwrap());
            }
        };

        if self.config.clone().get_nip05().is_some() {
            metadata = metadata.nip05(self.config.clone().get_nip05().unwrap());
        };

        if self.config.clone().get_lud16().is_some() {
            metadata = metadata.lud16(self.config.lud16.clone().unwrap());
        };

        debug!("{:?}", metadata);

        // Broadcast metadata (NIP-01) to relays
        let profile_result = self.get_client().set_metadata(metadata).await.unwrap();

        Ok(profile_result)
    }

    // Add a relay in the current client instance
    pub async fn add_relay(&self, url: &str, proxy: Option<SocketAddr>) -> Result<(), NostrError> {
        self.client.add_relay(url, proxy).await
    }
    // Remove a relay in the current client instance
    pub async fn remove_relay(&self, url: &str) -> Result<(), NostrError> {
        self.client.remove_relay(url).await
    }

    // Broadcasts message (NIP-02) to nostr relays
    pub async fn publish(self, _message: String) -> Result<()> {
        //  self.client.send_client_msg(message).await;
        Ok(())
    }

    // Get current client instance
    pub fn get_client(&self) -> &Client {
        &self.client
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::from_filename;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_new_nostr_default_instance() {
        from_filename(".env.test").ok();

        let profile = Profile {
            ..Default::default()
        };

        let client = NostrInstance::new(profile).await;

        assert_eq!(
            client.config.display_name,
            Some("satoshi-nakamoto".to_string())
        );
        assert_eq!(client.config.id, "default".to_string());
        assert_eq!(
            client.config.private_key,
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string()
        );

        assert_eq!(
            client.client.keys().public_key().to_string(),
            "4646ae5047316b4230d0086c8acec687f00b1cd9d1dc634f6cb358ac0a9a8fff".to_string()
        );
    }

    #[tokio::test]
    async fn test_new_nostr_instance_with_custom_profile() {
        from_filename(".env.test").ok();

        let profile = Profile {
            id: "Hal-Finney".to_string(),
            display_name: Some("Hal Finney".to_string()),
            about: Some("Running Bitcoin".to_string()),
            private_key: "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
                .to_string(),
            ..Default::default()
        };

        let client = NostrInstance::new(profile).await;

        assert_eq!(client.config.display_name, Some("Hal Finney".to_string()));
        assert_eq!(client.config.id, "Hal-Finney".to_string());
        assert_eq!(
            client.config.private_key,
            "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789".to_string()
        );

        assert_eq!(
            client.client.keys().public_key().to_string(),
            "4deb5e4bf849790657361d0559b96d9277fdfcf02f6f78f021e834b7282c9db8".to_string()
        );
    }

    #[tokio::test]
    async fn test_custom_relays_load() {
        from_filename(".env.test").ok();
        let mut relays = Vec::new();

        relays.push(Relay {
            name: "test".to_string(),
            target: "ws://umbrel.local".to_string(),
            proxy: None,
            active: true,
        });

        let profile = Profile {
            id: "Hal-Finney".to_string(),
            display_name: Some("Hal Finney".to_string()),
            about: Some("Running Bitcoin".to_string()),
            private_key: "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
                .to_string(),
            relays,
            ..Default::default()
        };

        let client = NostrInstance::new(profile).await;

        let relays = client.client.relays().await;
        let url = Url::from_str("ws://umbrel.local").unwrap();
        let relay = &relays[&url];

        assert_eq!(relay.url().as_str(), "ws://umbrel.local/");
    }

    #[tokio::test]
    async fn test_client_add_relay() {
        from_filename(".env.test").ok();

        let profile = Profile {
            id: "Hal-Finney".to_string(),
            display_name: Some("Hal Finney".to_string()),
            about: Some("Running Bitcoin".to_string()),
            private_key: "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
                .to_string(),
            ..Default::default()
        };

        let client = NostrInstance::new(profile).await;

        let original_size = &client.client.relays().await.keys().len();

        assert_eq!(original_size, &0);

        let _ = &client.add_relay("ws://umbrel.local", None).await;

        let new_size = client.client.relays().await.keys().len();

        assert_eq!(new_size, 1);
    }

    #[tokio::test]
    async fn test_client_remove_relay() {
        from_filename(".env.test").ok();

        let mut relays = Vec::new();

        relays.push(Relay {
            name: "test".to_string(),
            target: "ws://umbrel.local".to_string(),
            proxy: None,
            active: true,
        });

        let profile = Profile {
            id: "Hal-Finney".to_string(),
            display_name: Some("Hal Finney".to_string()),
            about: Some("Running Bitcoin".to_string()),
            private_key: "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
                .to_string(),
            relays,
            ..Default::default()
        };

        let client = NostrInstance::new(profile).await;

        let original_size = &client.client.relays().await.keys().len();

        assert_eq!(original_size, &1);

        let url = "ws://umbrel.local/";
        let _ = &client.remove_relay(url).await;

        let new_size = &client.client.relays().await.keys().len();

        assert_eq!(new_size, &0);
    }
}
