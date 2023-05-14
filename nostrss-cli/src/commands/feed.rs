#![allow(dead_code)]

use std::str::FromStr;

use clap::{Parser, ValueEnum};
use nostrss_grpc::grpc::{
    nostrss_grpc_client::NostrssGrpcClient, AddFeedRequest, FeedInfoRequest, FeedItem,
    FeedsListRequest,
};
use tabled::{Table, Tabled};
use tonic::{async_trait, transport::Channel};
use url::Url;

use super::CommandsHandler;

#[derive(Clone, PartialEq, Parser, Debug, ValueEnum)]
pub enum FeedActions {
    Add,
    Delete,
    List,
    Info,
}

pub struct FeedCommandsHandler {
    pub client: NostrssGrpcClient<Channel>,
}

#[derive(Tabled)]
pub struct FeedDetailsTemplate {
    pub key: String,
    pub value: String,
}

pub struct FullFeedTemplate {
    pub id: String,
    pub name: String,
    pub url: String,
    pub schedule: String,
    pub profiles: String,
    pub tags: String,
    pub template: String,
    pub cache_size: String,
    pub pow_level: String,
}

impl From<FeedItem> for FullFeedTemplate {
    fn from(value: FeedItem) -> Self {
        let profiles = value.profiles.join(",");
        let tags = value.tags.join(",");
        let cache_size = value.cache_size.to_string();
        let pow_level = value.pow_level.to_string();
        Self {
            id: value.id,
            name: value.name,
            url: value.url,
            schedule: value.schedule,
            profiles: profiles,
            tags: tags,
            template: value.template.unwrap_or("".to_string()),
            cache_size: cache_size,
            pow_level: pow_level,
        }
    }
}

impl FullFeedTemplate {
    fn properties_to_vec(&self) -> Vec<FeedDetailsTemplate> {
        let properties: Vec<(String, &String)> = [
            ("id".to_string(), &self.id),
            ("name".to_string(), &self.name),
            ("url".to_string(), &self.url),
            ("schedule".to_string(), &self.schedule),
            ("profiles".to_string(), &self.profiles),
            ("tags".to_string(), &self.tags),
            ("template".to_string(), &self.template),
            ("cache_size".to_string(), &self.cache_size),
            ("pow_level".to_string(), &self.pow_level),
        ]
        .to_vec();

        properties
            .into_iter()
            .map(|p| FeedDetailsTemplate {
                key: p.0,
                value: p.1.to_string(),
            })
            .collect()
    }
}

#[derive(Tabled)]
struct FeedsTemplate {
    name: String,
    url: String,
    schedule: String,
}

impl FeedsTemplate {
    fn new(name: &str, url: &str, schedule: &str) -> Self {
        Self {
            name: name.to_string(),
            url: url.to_string(),
            schedule: schedule.to_string(),
        }
    }
}

#[async_trait]
impl CommandsHandler for FeedCommandsHandler {}

impl FeedCommandsHandler {
    pub async fn handle(&mut self, action: FeedActions) {
        match action {
            FeedActions::Add => self.add().await,
            FeedActions::Delete => self.delete(),
            FeedActions::List => self.list().await,
            FeedActions::Info => self.info().await,
        }
    }

    async fn list(&mut self) {
        let request = tonic::Request::new(FeedsListRequest {});
        let response = self.client.feeds_list(request).await;
        match response {
            Ok(response) => {
                let raws: Vec<FeedsTemplate> = response
                    .into_inner()
                    .feeds
                    .into_iter()
                    .map(|f| FeedsTemplate::new(&f.id, &f.url, &f.schedule))
                    .collect();

                let table = Table::new(raws).to_string();
                println!("{}", table);
            }

            Err(e) => {
                println!("Error {}: {}", e.code(), e.message());
            }
        }
    }

    async fn add(&mut self) {
        println!("=== Add a new feed ===");
        let id = self.get_input("Id: ", Some(Self::required_input_validator));
        let name = self.get_input("Name: ", Some(Self::required_input_validator));
        let url = self.get_input("Url: ", Some(Self::url_validator));
        let schedule = self.get_input("scheduler pattern: ", Some(Self::cron_pattern_validator));
        let profiles: Vec<String> = self
            .get_input("profiles ids (separated with coma): ", None)
            .split(",")
            .into_iter()
            .map(|e| e.trim().to_string())
            .collect();
        let tags: Vec<String> = self
            .get_input("Tags (separated with coma):", None)
            .split(",")
            .into_iter()
            .map(|e| e.trim().to_string())
            .collect();
        let template = self.get_input("Template path: ", None);
        let cache_size = self.get_input("Cache size:", None).parse().unwrap_or(100);
        let pow_level = self.get_input("Pow Level", None).parse().unwrap_or(0);

        let request = tonic::Request::new(AddFeedRequest {
            id,
            name,
            url,
            schedule,
            profiles: profiles,
            template: Some(template),
            tags,
            cache_size,
            pow_level,
        });

        let response = self.client.add_feed(request).await;

        match response {
            Ok(response) => {
                println!("Feed successfuly added");
            }
            Err(e) => {
                println!("Error: {}: {}", e.code(), e.message());
            }
        }
    }

    fn delete(&self) {
        println!("=== Remove a new relay ===");
    }

    async fn info(&mut self) {
        let id = self.get_input("Id: ", None);

        let request = tonic::Request::new(FeedInfoRequest {
            id: id.trim().to_string(),
        });
        let response = self.client.feed_info(request).await;

        match response {
            Ok(response) => {
                let feed = response.into_inner().feed;

                let feed = FullFeedTemplate::from(feed);

                let table = Table::new(feed.properties_to_vec());
                println!("{}", table.to_string());

                // println!("No feed found for this id");
            }
            Err(e) => {
                println!("Error {}: {}", e.code(), e.message());
            }
        }
    }

    fn required_input_validator(value: String) -> bool {
        if value.len() == 0 {
            return false;
        }

        return true;
    }

    fn url_validator(value: String) -> bool {
        let r = Url::parse(&value);

        match r {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn cron_pattern_validator(value: String) -> bool {
        let r = cron::Schedule::from_str(&value);

        match r {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}
