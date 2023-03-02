use std::collections::HashMap;

use clap::Parser;

use crate::{nostr::config::NostrConfig, rss::config::RssConfig};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// path to the relays list to load on init
    #[arg(long)]
    pub relays: Option<String>,

    /// path to the feeds list to load on init
    #[arg(long)]
    pub feeds: Option<String>,

    /// The p   rivate key to populate keys
    #[arg(long)]
    pub private_key: Option<String>,
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub nostr_config: NostrConfig,
    pub rss_config: RssConfig,
    pub rss_index: HashMap<String, String>,
}

impl AppConfig {
    pub fn new() -> Self {
        let args = Args::parse();

        let nostr_config = NostrConfig::new(&args);
        let rss_config: RssConfig = RssConfig::new(&args);
        let map: HashMap<String, String> = HashMap::new();
        Self {
            nostr_config,
            rss_config,
            rss_index: map,
        }
    }

    pub fn update_rss_map(mut self, new_map: HashMap<String, String>) {
        self.rss_index = new_map;
    }
}
