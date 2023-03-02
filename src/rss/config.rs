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
    use std::io::Write;

    use super::*;
    use tempfile::NamedTempFile;

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

    #[test]
    fn test_load_yaml_feeds_valid() {
        // Create a temporary YAML file with valid contents
        let feeds_yaml = r#"
        - name: "RSS Feed 1"
          url: "https://rss-feed-1.com/feed"
          schedule: "*/5 * * * *"
        - name: "RSS Feed 2"
          url: "https://rss-feed-2.com/feed"
          schedule: "0 */2 * * *"
        "#;
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(feeds_yaml.as_bytes()).unwrap();

        // Load the feeds from the file and assert they were loaded correctly
        let config = RssConfig::new(&Args {
            feeds: Some(file.path().to_str().unwrap().to_string()),
            relays: None,
            private_key: None,
        });
        let expected_feeds = vec![
            Feed {
                name: "RSS Feed 1".to_string(),
                url: "https://rss-feed-1.com/feed".parse().unwrap(),
                schedule: "*/5 * * * *".to_string(),
            },
            Feed {
                name: "RSS Feed 2".to_string(),
                url: "https://rss-feed-2.com/feed".parse().unwrap(),
                schedule: "0 */2 * * *".to_string(),
            },
        ];
        assert_eq!(config.feeds, expected_feeds);
    }
}

// #[test]
// fn test_load_yaml_feeds() {
//     let args = Args { feeds: Some("tests/fixtures/valid_yaml_feeds.yml".to_string()), ..Default::default() };
//     let config = RssConfig::new(&args);
//     assert_eq!(config.feeds.len(), 2);

//     let expected_feeds = vec![
//         Feed {
//             name: "RSS Feed 1".to_string(),
//             url: "https://rss-feed-1.com/feed".parse().unwrap(),
//             schedule: "*/5 * * * *".to_string(),
//         },
//         Feed {
//             name: "RSS Feed 2".to_string(),
//             url: "https://rss-feed-2.com/feed".parse().unwrap(),
//             schedule: "0 */2 * * *".to_string(),
//         },
//     ];

//     assert_eq!(config.feeds, expected_feeds);
// }

// #[test]
// fn test_load_json_feeds() {
//     let args = Args { feeds: Some("tests/fixtures/valid_json_feeds.json".to_string()), ..Default::default() };
//     let config = RssConfig::new(&args);
//     assert_eq!(config.feeds.len(), 2);

//     let expected_feeds = vec![
//         Feed {
//             name: "RSS Feed 1".to_string(),
//             url: "https://rss-feed-1.com/feed".parse().unwrap(),
//             schedule: "*/5 * * * *".to_string(),
//         },
//         Feed {
//             name: "RSS Feed 2".to_string(),
//             url: "https://rss-feed-2.com/feed".parse().unwrap(),
//             schedule: "0 */2 * * *".to_string(),
//         },
//     ];

//     assert_eq!(config.feeds, expected_feeds);
// }
