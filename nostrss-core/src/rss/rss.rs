#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;

use super::config::Feed;
use super::config::RssConfig;
use tokio_cron_scheduler::Job;
use tokio_cron_scheduler::JobScheduler;
use uuid::Uuid;

#[derive(Clone)]
pub struct RssInstance {
    pub config: RssConfig,
    pub scheduler: Arc<JobScheduler>,
    pub feeds_jobs: HashMap<String, Uuid>,
    pub feeds: Vec<Feed>,
    pub maps: HashMap<String, HashMap<String, String>>,
}

impl RssInstance {
    pub async fn new(config: RssConfig) -> Self {
        let scheduler = match JobScheduler::new().await {
            Ok(result) => Arc::new(result),
            Err(_) => {
                // We shall improve the job creation error in a better way than just a panic
                panic!("Job creation error. Panicking !");
            }
        };
        let feeds = config.feeds.clone();
        let feeds_jobs = HashMap::new();
        Self {
            config,
            scheduler,
            feeds_jobs,
            feeds,
            maps: HashMap::new(),
        }
    }

    // Add a job to the scheduler.
    // Might be useless as the scheduler is publicly accessible.
    pub async fn add_job(self, job: Job) {
        let scheduler = self.scheduler;

        _ = scheduler.add(job).await;
    }

    // Remove a job to the scheduler.
    // Might be useless as the scheduler is publicly accessible.
    pub async fn remove_job(self, uuid: uuid::Uuid) {
        let scheduler = self.scheduler;

        _ = scheduler.remove(&uuid).await;
    }

    pub fn get_feeds(&self) -> Vec<Feed> {
        self.feeds.clone()
    }
}
