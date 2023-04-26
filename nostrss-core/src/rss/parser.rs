#![allow(dead_code)]

use feed_rs::model::{Entry, Feed as RemoteFeed};
use log::info;
use std::error::Error;
use std::fmt;

// pub struct Feed {
//     url: String,
//     delay: i32,
// }

/// RSS parsing processor
pub struct RssParser {}

impl RssParser {
    // Reads a remote RSS feed.
    pub async fn read(url: String) -> Result<RemoteFeed, RssParserError> {
        info!("requesting {:?}", url);

        // fetch
        let request_response = match reqwest::get(url).await {
            Ok(value) => value,
            Err(_) => {
                return Err(RssParserError::new("Error while fetching Rss Feed"));
            }
        };

        // read
        let content = match request_response.text().await {
            Ok(result) => result,
            Err(_) => {
                return Err(RssParserError::new("Error while reading Rss feed response"));
            }
        };

        // parse
        let feed = match feed_rs::parser::parse(content.as_bytes()) {
            Ok(feed) => feed,
            Err(_) => {
                return Err(RssParserError::new("Error while parsing Rss feed stream"));
            }
        };

        Ok(feed)
    }

    // Retrieves the first item from a remote feed
    pub async fn get_first_item(url: String) -> Result<Entry, RssParserError> {
        let feed = match Self::read(url).await {
            Ok(feed) => feed,
            Err(e) => {
                return Err(e);
            }
        };

        Ok(feed.entries[0].clone())
    }

    // Retrieves all items from a remote feed
    pub async fn get_items(url: String) -> Result<Vec<Entry>, RssParserError> {
        let feed = match Self::read(url).await {
            Ok(feed) => feed,
            Err(e) => {
                return Err(e);
            }
        };

        Ok(feed.entries)
    }

    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug)]
pub struct RssParserError {
    pub message: String,
}

impl RssParserError {
    pub fn new(message: &str) -> RssParserError {
        RssParserError {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for RssParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MyError: {}", self.message)
    }
}

impl Error for RssParserError {}
