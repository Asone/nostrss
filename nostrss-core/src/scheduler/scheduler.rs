use feed_rs::model::Entry;
use log::{debug, error};
use nostr_sdk::{prelude::FromSkStr, Client, EventBuilder, Keys, Kind, Tag};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, MutexGuard};
use tokio_cron_scheduler::Job;

use crate::{
    nostr::relay::Relay,
    profiles::config::Profile,
    rss::{config::Feed, parser::RssParser},
    template::template::TemplateProcessor,
};

/// Cronjob creation method
pub async fn schedule(
    rule: &str,
    feed: Feed,
    map: Arc<Mutex<HashMap<String, Vec<String>>>>,
    client: Arc<Mutex<Client>>,
    profiles: Arc<Mutex<HashMap<String, Profile>>>,
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

        let map_arc = Arc::clone(&map_job_copy);
        let profiles_arc = Arc::clone(&profiles);
        let client_arc = Arc::clone(&client);
        Box::pin(async move {
            let mut map_lock = map_arc.lock().await;
            let feed = feed.clone();
            let uuid = &uuid.to_string();
            let mut map = map_lock[uuid].clone();

            let client_lock = client_arc.lock().await;

            let profiles_lock = profiles_arc.lock().await;

            match RssParser::get_items(feed.url.to_string()).await {
                Ok(entries) => {
                    // Calls the method that
                    RssNostrJob::process(
                        feed.clone(),
                        profile_ids,
                        entries,
                        &mut map,
                        client_lock,
                        profiles_lock,
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
    });

    let job = match job {
        Ok(j) => j,
        Err(e) => {
            println!("{:?}", e);
            panic!()
        }
    };

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
    pub async fn _client_prepare(
        _client: MutexGuard<'_, Client>,
        _profile: MutexGuard<'_, Profile>,
    ) {
    }

    pub async fn _client_clean(_client: Client) {}
    pub async fn process(
        feed: Feed,
        profile_ids: Vec<String>,
        entries: Vec<Entry>,
        map: &mut Vec<String>,
        client: MutexGuard<'_, Client>,
        profiles_lock: MutexGuard<'_, HashMap<String, Profile>>,
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

                    let mut tags = Self::get_tags(&feed.tags);

                    // Declare NIP-48.
                    tags.push(Self::get_nip48(&tags, feed.id.clone()));

                    let message = match TemplateProcessor::parse(feed.clone(), entry.clone()) {
                        Ok(message) => message,
                        Err(e) => {
                            // make tick fail in non-critical way
                            error!("{}", e);
                            return;
                        }
                    };

                    for profile_id in &profile_ids {
                        let profile = profiles_lock.get(profile_id);

                        if profile.is_none() {
                            error!(
                                "Profile {} for stream {} not found. Job skipped.",
                                profile_id, feed.name
                            );
                            return;
                        }

                        let profile = profile.unwrap();

                        let keys = match Keys::from_sk_str(profile.private_key.as_str()) {
                            Ok(val) => val,
                            Err(e) => {
                                println!("{:?}", e);
                                // warn!("Invalid private key found for Nostr. Generating random keys...");
                                panic!("Invalid private key found. This should not happen.");
                            }
                        };

                        // _ = RssNostrJob::client_prepare(client,profile).await;

                        let recommended_relays_ids =
                            profile.recommended_relays.clone().unwrap_or(Vec::new());
                        let mut recommended_relays_tags = Self::get_recommended_relays(
                            recommended_relays_ids,
                            &profile.relays.clone(),
                        );

                        _ = &tags.append(&mut recommended_relays_tags);

                        let event = EventBuilder::new(nostr_sdk::Kind::TextNote, &message, &tags)
                            .to_pow_event(&keys, profile.pow_level);

                        match event {
                            Ok(e) => match client.send_event(e).await {
                                Ok(event_id) => log::info!("Entry published with id {}", event_id),
                                Err(e) => log::error!("Error publishing entry : {}", e),
                            },
                            Err(_) => panic!("Note couldn't be sent"),
                        };

                        // _ = RssNostrJob::client_clean(client,profile).await;
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

    fn get_nip48(mut tags: &Vec<Tag>, feed_id: String) -> Tag {
        // Declare NIP-48.
        // NIP-48 : declares to be a proxy from an external signal (rss,activityPub)
        Tag::Proxy {
            id: feed_id,
            protocol: nostr_sdk::prelude::Protocol::Rss,
        }
    }

    fn get_recommended_relays(recommended_relays_ids: Vec<String>, relays: &[Relay]) -> Vec<Tag> {
        let mut relay_tags = Vec::new();
        for relay_name in recommended_relays_ids {
            let r = relays.iter().find(|relay| relay.name == relay_name);
            if r.clone().is_none() {
                continue;
            }

            let tag = Tag::RelayMetadata(r.unwrap().target.clone().into(), None);
            relay_tags.push(tag);
        }

        relay_tags
    }
}

#[cfg(test)]
mod tests {

    use nostr_sdk::prelude::TagKind;

    use super::*;

    #[test]
    fn test_nip_48_signal() {}

    #[test]
    fn test_get_tags() {
        let relay_ids = ["test".to_string()].to_vec();
        let relays = [
            Relay {
                name: "test".to_string(),
                target: "wss://nostr.up".to_string(),
                active: true,
                proxy: None,
                pow_level: 0,
            },
            Relay {
                name: "mushroom".to_string(),
                target: "wss://mushroom.dev".to_string(),
                active: true,
                proxy: None,
                pow_level: 0,
            },
        ]
        .to_vec();
        let tags = RssNostrJob::get_recommended_relays(relay_ids, &relays);

        let tag = tags[0].clone();
        assert_eq!(tag.kind(), TagKind::R);
        assert_eq!(tag.as_vec()[0], "r");
        assert_eq!(tag.as_vec()[1], "wss://nostr.up");
    }

    #[test]
    fn test_nip_48() {
        let feed_id = "https://www.test.com";
        let mut tags: Vec<Tag> = [].to_vec();
        let nip_48 = RssNostrJob::get_nip48(&tags, feed_id.clone().to_string());

        assert_eq!(nip_48.kind(), TagKind::Proxy);
    }

    #[test]
    fn test_recommended_relays() {
        let feed_tags = ["ad".to_string(), "lorem".to_string(), "ipsum".to_string()].to_vec();
        let tags = RssNostrJob::get_tags(&Some(feed_tags));

        assert_eq!(tags.len(), 3);
        let tag = tags[0].clone();
        assert_eq!(tag.kind(), TagKind::T);
        assert_eq!(tag.as_vec()[0], "t");
        assert_eq!(tag.as_vec()[1], "ad");
    }
}
