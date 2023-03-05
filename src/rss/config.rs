use std::path::Path;

use log::{error, info};
use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::config::Args;

#[derive(Debug)]
pub enum RssConfigErrors {
    FileLocationError,
    FileFormatError,
    FileParsingError,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Feed {
    pub id: String,
    pub name: String,
    pub url: Url,
    pub schedule: String,
}

#[derive(Debug, Clone)]
pub struct RssConfig {
    pub feeds: Vec<Feed>,
}

impl RssConfig {
    pub fn new(args: &Args) -> Self {
        let mut config = Self { feeds: Vec::new() };

        if args.feeds.is_some() {
            info!("Found Rss file path argument. Parsing file...");
            config = config.load_feeds(&args.feeds.as_ref().unwrap());
        }

        config
    }

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
        let args = Args {
            feeds: Some("./src/fixtures/rss.json".to_string()),
            relays: None,
            private_key: None,
        };

        let config = RssConfig::new(&args);
        assert_eq!(config.feeds.len(), 4);
    }

    #[test]
    fn test_load_valid_yaml_feed() {
        let args = Args {
            feeds: Some("./src/fixtures/rss.yaml".to_string()),
            relays: None,
            private_key: None,
        };

        let config = RssConfig::new(&args);
        assert_eq!(config.feeds.len(), 3);
    }

    #[test]
    fn test_load_feeds_invalid_path() {
        let args = Args {
            feeds: Some("invalid_path.json".to_string()),
            relays: None,
            private_key: None,
        };
        let config = RssConfig::new(&args);
        assert_eq!(config.feeds.len(), 0);
    }

    #[test]
    fn test_load_feeds_invalid_extension() {
        let args = Args {
            feeds: Some("invalid_path.text".to_string()),
            relays: None,
            private_key: None,
        };

        let config = RssConfig::new(&args);
        assert_eq!(config.feeds.len(), 0);
    }
}
