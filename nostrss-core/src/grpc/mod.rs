pub mod feed_request;
pub mod grpc_service;
pub mod profile_request;

#[cfg(test)]
mod grpctest_utils {
    use std::{collections::HashMap, sync::Arc};

    use dotenv::from_filename;
    use tokio::sync::Mutex;

    use crate::{
        app::app::App,
        nostr::nostr::NostrInstance,
        profiles::config::Profile,
        rss::{config::RssConfig, rss::RssInstance},
        scheduler::scheduler::schedule,
    };

    pub async fn mock_app() -> App {
        from_filename(".env.test").ok();
        let rss_path = Some("./src/fixtures/rss.yaml".to_string());
        let rss_config = RssConfig::new(rss_path);

        let rss = RssInstance::new(rss_config).await;

        let default_profile = Profile {
            ..Default::default()
        };

        let test_profile = Profile {
            id: "test".to_string(),
            ..Default::default()
        };

        let mut profiles = HashMap::new();

        profiles.insert(default_profile.id.clone(), default_profile);
        profiles.insert(test_profile.id.clone(), test_profile);

        let mut clients = HashMap::new();

        for profile in profiles.clone() {
            let client = NostrInstance::new(profile.1).await;
            clients.insert(profile.0.clone(), client);
        }

        let scheduler = tokio_cron_scheduler::JobScheduler::new().await.unwrap();
        let mut app = App {
            rss,
            scheduler: Arc::new(scheduler),
            clients,
            profiles: profiles,
            feeds_jobs: HashMap::new(),
            feeds_map: HashMap::new(),
        };

        for feed in app.rss.feeds.clone() {
            let job = schedule(
                feed.clone().schedule.as_str(),
                feed.clone(),
                Arc::new(Mutex::new(app.feeds_map.clone())),
                Arc::new(Mutex::new(app.clients.clone())),
            )
            .await;

            _ = &app.rss.feeds_jobs.insert(feed.id, job.guid());
        }

        app
    }
}
