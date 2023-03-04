use log::{error, info};
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
    map: Arc<Mutex<HashMap<String, Vec<String>>>>,
    client: Arc<Mutex<NostrInstance>>,
) -> Job {
    // Create a copy of the map arc that will be solely used into the job
    let map_job_copy = Arc::clone(&map);

    let job_feed = feed.clone();
    let job = Job::new_async(rule, move |uuid, _lock| {
        // Copy feed for job execution
        let feed = job_feed.clone();

        // Arc instances for current job
        let client_arc = Arc::clone(&client);
        let map_arc = Arc::clone(&map_job_copy);
        return Box::pin(async move {
            let mut map_lock = map_arc.lock().await;

            let uuid = &uuid.to_string();
            let mut map = map_lock[uuid].clone();

            match RssParser::get_items(feed.url.to_string()).await {
                Ok(entries) => {
                    let client_lock = client_arc.lock().await;
                    for entry in entries {
                        let title = match entry.title {
                            Some(title_text) => title_text.content,
                            None => "".to_string(),
                        };

                        let url = &entry.links[0].href;
                        let entry_id = &entry.id;
                        match &map.contains(entry_id) {
                            true => {
                                info!(
                                    "Found entry for {} on feed with id {}, skipping publication.",
                                    entry_id, &feed.id
                                );
                            }
                            false => {
                                info!(
                                    "Entry not found for {} on feed with id {}, publishing...",
                                    entry_id, &feed.id
                                );
                                let message = format!("{} - {}. Url: {}", &feed.name, title, url);
                                let _ = client_lock.send_message(&message).await;
                                map.insert(0, entry.id);
                            }
                        }
                    }

                    // Remove old entries if the vec has over 200 elements
                    // The limit of entries should be provided dynamicaly in further
                    // iterations
                    map.truncate(200);
                    let _ = &map_lock.insert(uuid.to_string(), map);
                }
                Err(_) => {
                    error!(
                        "Error while parsing RSS stream for feed with {} id. Skipping...",
                        feed.id
                    );
                }
            };
        });
    })
    .unwrap();

    let f = feed.clone();

    // Initialize the Vec that will store the retained entries of feed for current feed.
    let mut map_lock = map.lock().await;
    let initial_snapshot = feed_snapshot(f).await;
    map_lock.insert(job.guid().to_string(), initial_snapshot);

    return job;
}

// Retrieves a feed and returns a vec of ids for the feed.
// This method is used to provide initial snapshot of the rss feeds
// In order to avoid to spam relays with initial rss feed fetch.
pub async fn feed_snapshot(feed: Feed) -> Vec<String> {
    let mut vec = Vec::new();
    match RssParser::get_items(feed.url.to_string()).await {
        Ok(entries) => {
            for entry in entries {
                vec.push(entry.id)
            }
        }
        Err(_) => {
            error!(
                "Error while parsing RSS stream for feed with {} id. Skipping initial snapshot",
                feed.id
            );
        }
    };

    vec
}
