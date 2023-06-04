use std::{collections::HashMap, sync::Arc};

use crate::{
    nostr::service::NostrService,
    profiles::{config::Profile, profiles::ProfileHandler},
    rss::{
        config::{Feed, RssConfig},
        rss::RssInstance,
    },
};
use clap::Parser;
use log::info;
use nostr_sdk::{
    prelude::{FromSkStr, ToBech32},
    Client, Keys,
};

use tokio::sync::Mutex;
use tokio_cron_scheduler::JobScheduler;
use uuid::Uuid;

#[derive(Parser, Debug, Clone, Default)]
#[command(author, version, about, long_about = None)]
pub struct AppConfig {
    /// path to the relays list to load on init
    #[arg(long)]
    pub relays: String,

    /// path to the feeds list to load on init
    #[arg(long)]
    pub feeds: Option<String>,

    /// path to the profiles list to load
    #[arg(long)]
    pub profiles: Option<String>,

    /// The private key to populate keys
    #[arg(long)]
    pub private_key: Option<String>,
}

pub struct App {
    pub rss: RssInstance,
    pub scheduler: Arc<JobScheduler>,
    // pub clients: HashMap<String, NostrInstance>,
    pub feeds_jobs: HashMap<String, Uuid>,
    pub feeds_map: HashMap<String, Vec<String>>,
    pub nostr_service: NostrService,
    pub config: AppConfig,
    pub profile_handler: ProfileHandler,
}

impl App {
    pub async fn new(config: AppConfig) -> Self {
        let profile_handler = ProfileHandler::new(&config.profiles, &config.relays);

        let scheduler = match JobScheduler::new().await {
            Ok(result) => Arc::new(result),
            Err(_) => {
                // We shall improve the scheduler creation error in a better way than just a panic
                panic!("Scheduler init failure. Panicking !");
            }
        };

        // RSS feed handler
        let rss = RssInstance::new(RssConfig::new(config.clone().feeds)).await;

        let profiles = profile_handler.clone().get_profiles();

        let default_relays = profile_handler.clone().get_default_relays();

        for profile_entry in profiles {
            let profile_id = profile_entry.0.clone();
            let mut profile = profile_entry.1.clone();

            if profile.relays.is_empty() {
                profile.relays = default_relays.clone();
            }

            let keys = Keys::from_sk_str(profile.private_key.as_str()).unwrap();
            let profile_keys = &keys.public_key();

            info!(
                "public key for profile {}: {:?}",
                &profile_id.clone(),
                &profile_keys
            );
            info!(
                "bech32 public key : {:?}",
                &profile_keys.to_bech32().unwrap()
            );
        }

        let client = Client::new(&Keys::generate());

        for relay in default_relays.into_iter() {
            _ = &client.add_relay(relay.target, relay.proxy).await;
        }

        _ = &client.connect().await;

        let nostr_service =
            NostrService::new(client, config.relays.clone(), config.profiles.clone()).await;

        Self {
            rss,
            scheduler,
            feeds_jobs: HashMap::new(),
            feeds_map: HashMap::new(),
            nostr_service,
            config,
            profile_handler: ProfileHandler(HashMap::new()),
        }
    }

    pub async fn get_profiles(&self) -> Arc<Mutex<HashMap<String, Profile>>> {
        Arc::new(Mutex::new(self.nostr_service.profiles.clone()))
    }

    pub async fn update_profile_config(&self) -> bool {
        let profiles_arc = self.get_profiles().await;
        let profiles_lock = profiles_arc.lock().await;
        let profiles = profiles_lock
            .iter()
            .filter_map(|(_, profile)| {
                if profile.id.as_str() == "default" {
                    None
                } else {
                    Some(profile)
                }
            })
            .collect();

        if self.config.profiles.is_none() {
            return false;
        }

        let result = self
            .profile_handler
            .clone()
            .save_profiles(self.config.profiles.clone().unwrap().as_str(), profiles);

        result
    }

    pub async fn update_feeds_config(&self, feeds: &Vec<Feed>) -> bool {
        if self.config.feeds.is_none() {
            return false;
        }
        let rss = self
            .rss
            .config
            .clone()
            .save_feeds(&self.config.feeds.clone().unwrap(), feeds);
        rss
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::app::{app::App, test_utils};

    #[tokio::test]
    async fn update_profile_config_test() {
        let app = test_utils::mock_app().await;
    }
}
