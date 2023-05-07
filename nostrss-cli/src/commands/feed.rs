#![allow(dead_code)]

use nostrss_grpc::grpc::{nostrss_grpc_client::NostrssGrpcClient, FeedsListRequest};
use tabled::{Table, Tabled};
use tonic::{async_trait, transport::Channel};

use super::CommandsHandler;

pub struct FeedCommandsHandler {
    pub client: NostrssGrpcClient<Channel>,
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
impl CommandsHandler for FeedCommandsHandler {
    async fn handle(&mut self, action: String) {
        match action.as_str() {
            "add" => self.add(),
            "delete" => self.delete(),
            "list" => self.list().await,
            _ => {}
        }
    }
}

impl FeedCommandsHandler {
    async fn list(&mut self) {
        // Case logic should come here
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
                // for feed in response.into_inner().feeds {
                //     println!("{} : {}", feed.id, feed.url);
                // }
                let table = Table::new(raws).to_string();
                println!("{}", table);
            }
            // let table = Table::new(languages).to_string();
            Err(e) => {
                println!("Error {}: {}", e.code(), e.message());
            }
        }
    }

    fn add(&self) {
        println!("=== Add a new feed ===");
        let id = self.get_input("Id: ");
        let name = self.get_input("Name: ");
        let url = self.get_input("Url: ");
        let profiles = self.get_input("profiles ids (separated with coma): ");
        let schedule = self.get_input("scheduler pattern: ");

        println!(
            "{},{},{},{},{}",
            id.trim(),
            name.trim(),
            url.trim(),
            profiles.trim(),
            schedule.trim()
        );
    }

    fn delete(&self) {
        println!("=== Remove a new relay ===");
    }

    fn info(&self) {
        self.get_input("Id: ");
    }
}
