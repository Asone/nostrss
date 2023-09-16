use std::{collections::HashMap, fmt, sync::Arc};

use log::debug;
use nostr_sdk::{Client, EventBuilder, EventId, Keys, Metadata, Result};
use reqwest::Url;
use tokio::sync::Mutex;

use crate::{
    nostr::NostrProfile,
    profiles::{config::Profile, profiles::ProfileHandler},
};

use super::relay::Relay;

pub enum NostrServiceError {
    BroadcastError,
    ProfileNotFoundError,
}

impl fmt::Debug for NostrServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NostrServiceError::BroadcastError => write!(f, "Broadcast error"),
            NostrServiceError::ProfileNotFoundError => write!(f, "Profile not found"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NostrService {
    pub client: Client,
    pub default_relays: HashMap<Url, Relay>,
    pub profiles: HashMap<String, Profile>,
}

impl Default for NostrService {
    fn default() -> Self {
        Self {
            client: Client::new(&Keys::generate()),
            default_relays: HashMap::new(),
            profiles: HashMap::new(),
        }
    }
}

impl NostrService {
    pub async fn new(client: Client, relays: String, profiles: Option<String>) -> Self {
        let profile_handler = ProfileHandler::new(&profiles, &relays);

        let profiles = profile_handler.get_profiles();
        let default_relays = profile_handler.new_get_default_relays();
        Self {
            client,
            default_relays,
            profiles,
        }
    }

    pub async fn update_profile(&self, profile_id: String) -> Result<EventId, NostrServiceError> {
        let profile = match self.profiles.get(&profile_id) {
            Some(result) => result,
            None => return Err(NostrServiceError::ProfileNotFoundError),
        };

        let mut metadata = Metadata::new();

        if profile.clone().get_display_name().is_some() {
            // metadata.name(self.config.display_name.clone().unwrap());
            metadata = metadata.display_name(profile.clone().get_display_name().unwrap());
            metadata = metadata.name(profile.clone().get_name().unwrap());
        };

        if profile.clone().get_description().is_some() {
            metadata = metadata.about(profile.clone().get_description().unwrap());
        };

        if profile.clone().get_picture().is_some() {
            let parsed_url = nostr_sdk::Url::parse(profile.clone().get_picture().unwrap().as_str());

            if parsed_url.is_ok() {
                metadata = metadata.picture(parsed_url.unwrap());
            }
        };

        if profile.clone().get_banner().is_some() {
            let parsed_url = nostr_sdk::Url::parse(profile.clone().get_banner().unwrap().as_str());

            if parsed_url.is_ok() {
                metadata = metadata.banner(parsed_url.unwrap());
            }
        };

        if profile.clone().get_nip05().is_some() {
            metadata = metadata.nip05(profile.clone().get_nip05().unwrap());
        };

        if profile.clone().get_lud16().is_some() {
            metadata = metadata.lud16(profile.clone().get_lud16().unwrap());
        };

        debug!("{:?}", metadata);

        let event = EventBuilder::set_metadata(metadata)
            .to_event(&profile.get_keys())
            .unwrap();

        // Broadcast metadata (NIP-01) to relays
        let result = self.client.clone().send_event(event).await;

        if result.is_err() {
            return Err(NostrServiceError::BroadcastError);
        }

        Ok(result.unwrap())
    }

    pub async fn get_client(&self) -> Arc<Mutex<Client>> {
        Arc::new(Mutex::new(self.client.clone()))
    }
}
