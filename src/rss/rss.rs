use std::collections::HashMap;
use std::sync::Arc;

use super::config::RssConfig;
use tokio_cron_scheduler::Job;
use tokio_cron_scheduler::JobScheduler;
use uuid::Uuid;
#[derive(Clone)]
pub struct RssInstance {
    pub config: RssConfig,
    pub scheduler: Arc<JobScheduler>,
    pub feeds_jobs: HashMap<String, Uuid>,
}

impl RssInstance {
    pub async fn new(config: RssConfig) -> Self {
        let scheduler = match JobScheduler::new().await {
            Ok(result) => Arc::new(result),
            Err(e) => {
                panic!("meh")
            }
        };
        let feeds_jobs = HashMap::new();
        Self {
            config,
            scheduler,
            feeds_jobs,
        }
    }

    pub async fn add_job(self, job: Job) {
        let scheduler = self.scheduler;

        let _ = scheduler.add(job).await;
    }

    pub async fn remove_job(self, uuid: uuid::Uuid) {
        let scheduler = self.scheduler;

        let _ = scheduler.remove(&uuid).await;
    }
}
