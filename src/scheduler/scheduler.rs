use feed_rs::model::Text;
use log::info;
use md5;

use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;
use tokio_cron_scheduler::Job;

use crate::{
    nostr::nostr::NostrInstance,
    rss::{config::Feed, parser::RssParser},
};

/// Cronjob creation method
pub async fn schedule(
    rule: &str,
    feed: Feed,
    map: Arc<Mutex<HashMap<String, String>>>,
    client: Arc<Mutex<NostrInstance>>,
) -> Job {
    let job = Job::new_async(rule, move |uuid, _lock| {
        // Copy feed for job execution
        let feed = feed.clone();

        // Arc instances for current job
        let map_arc = Arc::clone(&map);
        let client_arc = Arc::clone(&client);

        return Box::pin(async move {
            match RssParser::get_first_item(feed.url.to_string()).await {
                // Get first item in rss feed/channel
                Ok(item) => {
                    // Locks the arcs to use shared instances and datas
                    let mut map = map_arc.lock().await;
                    let client_lock = client_arc.lock().await;

                    // Extract data from item for message publishing
                    let title = match item.title {
                        Some(title_text) => title_text.content,
                        None => "".to_string(),
                    };

                    let url = &item.links[0].href;

                    // Compute content checksum value
                    let content_hash = format!("{:?}", md5::compute(&title));

                    let default_content = "".to_string();

                    let previous_hash = map.get(&uuid.to_string()).unwrap_or(&default_content);

                    // Handle new items here
                    if previous_hash != &content_hash {
                        let message = format!("{:?}: {:?}. Url: {:?}", &feed.name, title, url);
                        let _ = client_lock.send_message(&message).await;
                        map.insert(uuid.to_string(), content_hash);
                    }

                    ()
                }
                Err(_) => {
                    info!("No item found for feed {:?}. Skipping", &feed.name);
                }
            }
        });
    })
    .unwrap();

    return job;
}
