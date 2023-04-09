#![allow(dead_code)]

use std::path::Path;

use log::{error, info};
use reqwest::Url;
use serde::{Deserialize, Serialize};

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
    pub url: Url,
    // The cronjob ticker rule for the feed job
    pub schedule: String,
    // The clients profiles to be used for publishing updates. Will use default profile if none provided
    pub profiles: Option<Vec<String>>,
    // The tags to be applied with the feed messages
    pub tags: Option<Vec<String>>,
    // The template path for publication
    pub template: Option<String>,
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
}

/// Builds a RSS config
#[derive(Debug, Clone)]
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
            Err(_) => {
                error!("Invalid Feed file");
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
            Err(_) => {
                error!("Invalid Feed file");
                return self;
            }
        };

        self.feeds = feeds;
        self
    }
}

#[cfg(test)]
pub mod tests {
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
}
