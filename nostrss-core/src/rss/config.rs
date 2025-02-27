#![allow(dead_code)]

use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{
    env::{self, VarError},
    fs::File,
    path::Path,
    str::FromStr,
};

#[derive(Debug)]
pub enum RssConfigErrors {
    LocationError,
    FormatError,
    ParsingError,
}

/// The [`Feed`] struct represents a feed as provided through
/// external file, be it either a `json` or a `yaml` file.
///
/// Examples of the struct is provided through in the [Fixtures](../fixtures/) folder.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Feed {
    // The id of the feed. Used for indexing in memory on runtime
    pub id: String,
    // The feed name to be displayed in the nostr messages
    pub name: String,
    // The URL to the RSS feed
    pub url: nostr_sdk::Url,
    // The cronjob ticker rule for the feed job
    pub schedule: String,
    // The clients profiles to be used for publishing updates. Will use default profile if none provided
    pub profiles: Option<Vec<String>>,
    // The tags to be applied with the feed messages
    pub tags: Option<Vec<String>>,
    // The template path for publication
    pub template: Option<String>,
    #[serde(default = "Feed::default_cache_size")]
    pub cache_size: Option<usize>,
    #[serde(default = "Feed::default_pow_level")]
    pub pow_level: u8,
}

impl Feed {
    // Retrieves the id of the feed
    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    // Retrieves the optional profile id of the feed
    pub fn get_profiles(&self) -> Option<Vec<String>> {
        self.profiles.clone()
    }

    // Checks if a feed uses a profile
    pub fn has_profile(&self, id: String) -> bool {
        if self.profiles.clone().is_none() {
            return false;
        }

        let profiles = &self.profiles.clone().unwrap();

        profiles.contains(&id)
    }

    // sets the tags for the feed
    fn set_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    pub fn default_cache_size() -> Option<usize> {
        match env::var("DEFAULT_CACHE_SIZE") {
            Ok(r) => Some(r.parse::<usize>().unwrap_or(1000)),
            Err(e) => None,
        }
        // let default_value = env::var("DEFAULT_CACHE_SIZE")
        //     .unwrap_or("100".to_string())
        //     .parse::<usize>()
        //     .unwrap_or(1000);
    }

    pub fn default_pow_level() -> u8 {
        env::var("DEFAULT_POW_LEVEL")
            .unwrap_or("0".to_string())
            .parse::<u8>()
            .unwrap_or(0)
    }
}

impl Default for Feed {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            name: "Generic feed".to_string(),
            url: nostr_sdk::Url::from_str("https://www.nostr.info").unwrap(),
            schedule: "0/10 * * * * *".to_string(),
            profiles: None,
            tags: Some(Vec::new()),
            template: None,
            cache_size: Self::default_cache_size(),
            pow_level: 0,
        }
    }
}
/// Builds a RSS config
#[derive(Debug, Clone, Default)]
pub struct RssConfig {
    pub feeds: Vec<Feed>,
}

impl RssConfig {
    // Builds a new instance of RSS feeds.
    // Takes an optional string that represents
    // the `path` to the feeds to load.
    pub fn new(path: Option<String>) -> Self {
        let mut config = Self { feeds: Vec::new() };

        if let Some(path) = path {
            info!("Found Rss file path argument. Parsing file...");
            config = config.load_feeds(&path);
        }

        config
    }

    // Loads the feeds into the [`RssConfig`] struct instance
    pub fn load_feeds(self, path: &str) -> Self {
        let path = Path::new(path);

        if path.is_file() {
            match path.extension() {
                Some(ext) => match ext.to_str() {
                    Some("yml") => {
                        return self.load_yaml_feeds(path);
                    }
                    Some("yaml") => {
                        return self.load_yaml_feeds(path);
                    }
                    Some("json") => {
                        return self.load_json_feeds(path);
                    }
                    _ => {
                        return self;
                    }
                },
                None => {
                    return self;
                }
            }
        }

        self
    }

    // Parses and serializes the config file when it is a `json` file
    pub fn load_json_feeds(mut self, path: &Path) -> Self {
        let file = match std::fs::File::open(path) {
            Ok(file) => file,
            Err(_) => {
                error!("Feeds file not found");
                return self;
            }
        };
        let feeds: Vec<Feed> = match serde_json::from_reader(file) {
            Ok(feeds) => feeds,
            Err(e) => {
                error!("Error parsing json feed file : {}", e);
                return self;
            }
        };

        self.feeds = feeds;
        self
    }

    // Parses and serializes the config file when it is a `yaml` file
    pub fn load_yaml_feeds(mut self, path: &Path) -> Self {
        let file = match std::fs::File::open(path) {
            Ok(file) => file,
            Err(_) => {
                error!("Feeds file not found");
                return self;
            }
        };
        let feeds: Vec<Feed> = match serde_yaml::from_reader(file) {
            Ok(feeds) => feeds,
            Err(e) => {
                error!("Error parsing yaml feed file : {}", e);
                return self;
            }
        };

        self.feeds = feeds;
        self
    }

    pub fn save_feeds(self, path: &str, feeds: &Vec<Feed>) -> bool {
        let path = Path::new(path);

        if path.is_file() {
            match path.extension() {
                Some(ext) => match ext.to_str() {
                    Some("yml") => {
                        return self.save_yaml_feeds(path, feeds);
                    }
                    Some("yaml") => {
                        return self.save_yaml_feeds(path, feeds);
                    }
                    Some("json") => {
                        return self.save_json_feeds(path, feeds);
                    }
                    _ => {
                        return false;
                    }
                },
                None => {
                    return false;
                }
            }
        }

        false
    }

    pub fn save_json_feeds(self, path: &Path, feeds: &Vec<Feed>) -> bool {
        // let serialized = serde_json::to_string(&feeds).unwrap();

        // let result = serde_json::to_writer_pretty(path, &feeds);

        let file = File::create(path).unwrap();
        let writer = std::io::BufWriter::new(file);
        let result = serde_json::to_writer_pretty(writer, &feeds);

        match result {
            Ok(_) => true,
            Err(e) => {
                error!("{}", e);
                false
            }
        }
    }

    pub fn save_yaml_feeds(self, path: &Path, feeds: &Vec<Feed>) -> bool {
        // let serialized = serde_json::to_string(&feeds).unwrap();

        // let result = serde_json::to_writer_pretty(path, &feeds);

        let file = File::create(path).unwrap();
        let writer = std::io::BufWriter::new(file);
        let result = serde_yaml::to_writer(writer, &feeds);

        match result {
            Ok(_) => true,
            Err(e) => {
                error!("{}", e);
                false
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::env::remove_var;

    use dotenv::from_filename;

    use super::*;

    #[test]
    fn test_load_valid_json_feed() {
        let path = Some("./src/fixtures/rss.json".to_string());

        let config = RssConfig::new(path);
        assert_eq!(config.feeds.len(), 3);
    }

    #[test]
    fn test_load_valid_yaml_feed() {
        let path = Some("./src/fixtures/rss.yaml".to_string());

        let config = RssConfig::new(path);
        assert_eq!(config.feeds.len(), 3);
    }

    #[test]
    fn test_load_feeds_invalid_path() {
        let path = Some("invalid_path.json".to_string());
        let config = RssConfig::new(path);
        assert_eq!(config.feeds.len(), 0);
    }

    #[test]
    fn test_load_feeds_invalid_extension() {
        let path = Some("invalid_path.text".to_string());

        let config = RssConfig::new(path);
        assert_eq!(config.feeds.len(), 0);
    }

    #[test]
    fn test_cache_size_behaviour_without_env_fallback() {
        remove_var("DEFAULT_CACHE_SIZE");
        let path = Some("./src/fixtures/rss.json".to_string());
        let config = RssConfig::new(path);

        assert_eq!(config.feeds[0].cache_size, Some(5));

        // Test undeclared cache size that should fall back to hard-coded cache value
        assert_eq!(config.feeds[1].cache_size, None);
    }

    #[test]
    fn test_cache_size_behaviour_with_env_fallback() {
        from_filename(".env.test").ok();
        let path = Some("./src/fixtures/rss.json".to_string());
        let config = RssConfig::new(path);

        // Test declared cache size
        assert_eq!(config.feeds[0].cache_size, Some(5));

        // Test undeclared cache size that should fall back on env var value
        assert_eq!(config.feeds[1].cache_size, Some(20));
    }
}
