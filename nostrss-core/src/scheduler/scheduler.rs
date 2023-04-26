use feed_rs::model::Entry;
use log::{debug, error};
use nostr_sdk::Tag;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, MutexGuard};
use tokio_cron_scheduler::Job;

use crate::{
    nostr::nostr::NostrInstance,
    rss::{config::Feed, parser::RssParser},
    template::template::TemplateProcessor,
};

/// Cronjob creation method
pub async fn schedule(
    rule: &str,
    feed: Feed,
    map: Arc<Mutex<HashMap<String, Vec<String>>>>,
    clients: Arc<Mutex<HashMap<String, NostrInstance>>>,
) -> Job {
    // Create a copy of the map arc that will be solely used into the job
    let map_job_copy = Arc::clone(&map);

    let job_feed = feed.clone();
    let job = Job::new_async(rule, move |uuid, _lock| {
        // Copy feed for job execution
        let feed = job_feed.clone();

        // Get the profiles ids associated to the feed for further use
        let profile_ids = feed
            .profiles
            .clone()
            .unwrap_or(["default".to_string()].to_vec());

        // Arc instances for current job
        let clients_arc = Arc::clone(&clients);
        let map_arc = Arc::clone(&map_job_copy);

        Box::pin(async move {
            let mut map_lock = map_arc.lock().await;
            let feed = feed.clone();
            let uuid = &uuid.to_string();
            let mut map = map_lock[uuid].clone();

            match RssParser::get_items(feed.url.to_string()).await {
                Ok(entries) => {
                    let clients_lock = clients_arc.lock().await;

                    // Calls the method that
                    RssNostrJob::process(
                        feed.clone(),
                        profile_ids,
                        entries,
                        &mut map,
                        clients_lock,
                    )
                    .await;

                    // Remove old entries if the vec has over 200 elements
                    // The limit of entries should be provided dynamicaly in further
                    // iterations.
                    map.truncate(feed.cache_size);
                    _ = &map_lock.insert(uuid.to_string(), map);
                }
                Err(_) => {
                    error!(
                        "Error while parsing RSS stream for feed with {} id. Skipping...",
                        feed.id
                    );
                }
            };
        })
    })
    .unwrap();

    let f = feed.clone();

    // Initialize the Vec that will store the retained entries of feed for current feed.
    // This avoids to spam the network on first fetch
    let mut map_lock = map.lock().await;
    let initial_snapshot = feed_snapshot(f).await;
    map_lock.insert(job.guid().to_string(), initial_snapshot);

    job
}

// Retrieves a feed and returns a vec of ids for the feed.
// This method is used to provide initial snapshot of the rss feeds
// In order to avoid to spam relays with initial rss feed fetch.
pub async fn feed_snapshot(feed: Feed) -> Vec<String> {
    let mut entries_snapshot = Vec::new();
    match RssParser::get_items(feed.url.to_string()).await {
        Ok(entries) => {
            for entry in entries {
                entries_snapshot.push(entry.id)
            }
        }
        Err(_) => {
            error!(
                "Error while parsing RSS stream for feed with {} id. Skipping initial snapshot",
                feed.id
            );
        }
    };

    entries_snapshot
}

pub struct RssNostrJob {}

impl RssNostrJob {
    pub async fn process(
        feed: Feed,
        profile_ids: Vec<String>,
        entries: Vec<Entry>,
        map: &mut Vec<String>,
        clients_lock: MutexGuard<'_, HashMap<String, NostrInstance>>,
    ) {
        for entry in entries {
            let entry_id = &entry.id;

            match &map.contains(entry_id) {
                true => {
                    debug!(
                        "Found entry for {} on feed with id {}, skipping publication.",
                        entry_id, &feed.id
                    );
                }
                false => {
                    debug!(
                        "Entry not found for {} on feed with id {}, publishing...",
                        entry_id, &feed.id
                    );

                    let tags = Self::get_tags(&feed.tags);

                    let message = match TemplateProcessor::parse(feed.clone(), entry.clone()) {
                        Ok(message) => message,
                        Err(e) => {
                            // make tick fail in non-critical way
                            error!("{}", e);
                            return ();
                        }
                    };

                    for profile_id in &profile_ids {
                        let client = clients_lock.get(profile_id);

                        if client.is_none() {
                            error!(
                                "No client found for this stream : {}. Job skipped.",
                                feed.name
                            );
                        }

                        if client.is_some() {
                            let client = client.unwrap();
                            match client.config.pow_level.clone() {
                                0 => client.send_message(&message, &tags).await,
                                _ => {
                                    client
                                        .send_pow_message(&message, &tags, client.config.pow_level)
                                        .await
                                }
                            }
                        }
                    }

                    map.insert(0, entry.id);
                }
            }
        }
    }

    fn get_tags(feed_tags: &Option<Vec<String>>) -> Vec<Tag> {
        let mut tags = Vec::new();

        if feed_tags.is_some() {
            for tag in feed_tags.clone().unwrap() {
                tags.push(Tag::Hashtag(tag.clone()));
            }
        }
        tags
    }
}
