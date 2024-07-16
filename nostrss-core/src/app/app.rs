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
use nostr_sdk::{prelude::RelayOptions, prelude::ToBech32, Client, Keys};

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

    /// Run the progam without broadcasting onto the network
    #[arg(long, action)]
    pub dry_run: bool,

    #[arg(long)]
    pub update: Option<bool>,
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

            let keys = Keys::parse(profile.private_key.as_str()).unwrap();
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
            let mut opts = RelayOptions::new();

            opts = opts.proxy(relay.proxy);

            _ = &client.add_relay_with_opts(relay.target, opts).await;
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

    pub async fn get_config(&self) -> Arc<Mutex<AppConfig>> {
        Arc::new(Mutex::new(self.config.clone()))
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
    use super::*;
    use crate::{app::test_utils, profiles::config::Profile};
    use std::env;
    use std::fs;

    fn prepare_test_files() {
        let current_dir = env::current_dir().unwrap();
        let workdir = current_dir.to_str().unwrap();

        let test_dir_path = format!("{}/{}", workdir.clone(), "src/fixtures/tests");

        let _ = fs::create_dir_all(test_dir_path.clone()).unwrap();

        let profiles_json_path = format!("{}/{}", workdir.clone(), "src/fixtures/profiles.json");
        let profiles_json_test_path = format!("{}/{}.test.json", test_dir_path.clone(), "profiles");
        fs::copy(profiles_json_path, profiles_json_test_path).unwrap();

        let profiles_yaml_path = format!("{}/{}", workdir.clone(), "src/fixtures/profiles.yaml");
        let profiles_yaml_test_path = format!("{}/{}.test.yaml", test_dir_path.clone(), "profiles");
        fs::copy(profiles_yaml_path, profiles_yaml_test_path).unwrap();

        let rss_json_path = format!("{}/{}", workdir.clone(), "src/fixtures/rss.json");
        let rss_json_test_path = format!("{}/{}.test.json", test_dir_path.clone(), "rss");
        fs::copy(rss_json_path, rss_json_test_path).unwrap();

        let rss_yaml_path = format!("{}/{}", workdir.clone(), "src/fixtures/rss.yaml");
        let rss_yaml_test_path = format!("{}/{}.test.yaml", test_dir_path.clone(), "rss");
        fs::copy(rss_yaml_path, rss_yaml_test_path).unwrap();
    }

    fn clean_test_files() {
        fs::remove_dir_all("src/fixtures/tests").unwrap();
    }

    #[tokio::test]
    async fn update_profile_config_test() {
        prepare_test_files();
        let mut app = test_utils::mock_app().await;

        let mut profiles = HashMap::new();

        let profile_1 = Profile {
            id: "test1".to_string(),
            ..Default::default()
        };

        let _ = &profiles.insert("test1".to_string(), profile_1);

        let profile_2 = Profile {
            id: "test2".to_string(),
            ..Default::default()
        };

        _ = &profiles.insert("test2".to_string(), profile_2);

        let profile_3 = Profile {
            id: "test3".to_string(),
            ..Default::default()
        };

        _ = &profiles.insert("test3".to_string(), profile_3);

        app.nostr_service.profiles = profiles;

        // Point app configuration to profiles json test file
        app.config.profiles = Some("src/fixtures/tests/profiles.test.json".to_string());

        let r = app.update_profile_config().await;
        assert_eq!(true, r);

        // Point app configuration to profiles yaml test file
        app.config.profiles = Some("src/fixtures/tests/profiles.test.yaml".to_string());

        let r = app.update_profile_config().await;
        assert_eq!(true, r);

        let feeds = [
            Feed {
                id: "test1".to_string(),
                ..Default::default()
            },
            Feed {
                id: "test2".to_string(),
                ..Default::default()
            },
            Feed {
                id: "test3".to_string(),
                ..Default::default()
            },
        ]
        .to_vec();

        app.config.feeds = Some("src/fixtures/tests/rss.test.yaml".to_string());

        let r = app.update_feeds_config(&feeds).await;
        assert_eq!(true, r);

        app.config.feeds = Some("src/fixtures/tests/rss.test.json".to_string());

        let r = app.update_feeds_config(&feeds).await;
        assert_eq!(true, r);

        clean_test_files()
    }
}
