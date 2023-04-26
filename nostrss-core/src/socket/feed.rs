use std::sync::Arc;

use tokio::sync::Mutex;

use crate::app::app::App;

pub struct FeedCommandHandler {}

impl FeedCommandHandler {
    pub async fn handle(app: Arc<Mutex<App>>, action: String) -> String {
        let res = match action.as_str() {
            "ADD" => Self::add(app).await,
            "DEL" => Self::delete(app).await,
            "LS" => Self::list(app).await,
            _ => "Unknown action".to_string(),
        };

        res
    }

    async fn add(app: Arc<Mutex<App>>) -> String {
        let _lock = app.lock().await;
        "Feed added".to_string()
    }

    async fn delete(app: Arc<Mutex<App>>) -> String {
        let _lock = app.lock().await;
        "Feed deleted".to_string()
    }

    async fn list(app: Arc<Mutex<App>>) -> String {
        let app_lock = match app.try_lock() {
            Ok(a) => a,
            Err(e) => {
                panic!("{}", e);
            }
        };
        let mut res = "Feeds list:".to_string();
        for feed in app_lock.rss.get_feeds() {
            res = format!("{}\n* {} : {}", res, feed.id, feed.url);
        }
        res
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use dotenv::from_filename;

    use super::*;
    use crate::{
        rss::{
            config::{Feed, RssConfig},
            rss::RssInstance,
        },
    };
    use tokio_cron_scheduler::JobScheduler;

    #[tokio::test]
    async fn test_socket_feed_list() {
        from_filename(".env.test").ok();
        let rss_config = RssConfig {
            feeds: [Feed {
                ..Default::default()
            }]
            .to_vec(),
        };

        let rss = RssInstance::new(rss_config).await;
        let scheduler = JobScheduler::new().await.unwrap();

        let app = App {
            rss,
            scheduler: Arc::new(scheduler),
            clients: HashMap::new(),
            profiles: HashMap::new(),
            feeds_jobs: HashMap::new(),
            feeds_map: HashMap::new(),
        };

        let result = FeedCommandHandler::list(Arc::new(Mutex::new(app))).await;
        let expected = "Feeds list:\n* default : https://www.nostr.info/";
        assert_eq!(result, expected);
    }
}
