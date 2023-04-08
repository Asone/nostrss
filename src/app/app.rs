use std::{collections::HashMap, sync::Arc};

use log::{info, warn};
use nostr_sdk::prelude::ToBech32;
use tokio_cron_scheduler::JobScheduler;

use crate::{
    nostr::nostr::NostrInstance,
    profiles::{config::Profile, profiles::ProfileHandler},
    rss::{config::RssConfig, rss::RssInstance},
};
use clap::Parser;
use uuid::Uuid;

#[derive(Parser, Debug)]
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
    pub clients: HashMap<String, NostrInstance>,
    pub profiles: HashMap<String, Profile>,
    pub feeds_jobs: HashMap<String, Uuid>,
    pub feeds_map: HashMap<String, Vec<String>>,
}

impl App {
    pub async fn new(config: AppConfig) -> Self {
        let profile_handler = ProfileHandler::new(config.profiles, config.relays);

        let scheduler = match JobScheduler::new().await {
            Ok(result) => Arc::new(result),
            Err(_) => {
                // We shall improve the scheduler creation error in a better way than just a panic
                panic!("Scheduler init failure. Panicking !");
            }
        };

        let rss = RssInstance::new(RssConfig::new(config.feeds)).await;

        let mut clients: HashMap<String, NostrInstance> = HashMap::new();

        // Create nostr clients based on profiles

        let profiles = profile_handler.clone().get_profiles();

        for profile_entry in profiles {
            let profile_id = profile_entry.0.clone();
            let mut profile = profile_entry.1.clone();

            if profile.relays.is_empty() {
                profile.relays = profile_handler.clone().get_default_relays();
            }

            let client = NostrInstance::new(profile).await;
            let profile_result = &client.update_profile().await;

            println!(
                "result of profile update for {} : {:?}",
                profile_id, profile_result
            );

            let profile_keys = &client.client.keys().public_key();
            clients.insert(profile_id.clone(), client);

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

        let feeds_jobs = HashMap::new();

        Self {
            rss,
            scheduler,
            clients,
            profiles: profile_handler.get_profiles(),
            feeds_jobs,
            feeds_map: HashMap::new(),
        }
    }
}
