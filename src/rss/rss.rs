use std::sync::Arc;

use super::config::RssConfig;
use tokio_cron_scheduler::JobScheduler;

#[derive(Clone)]
pub struct RssInstance {
    pub config: RssConfig,
    pub scheduler: Arc<JobScheduler>,
}

impl RssInstance {
    pub async fn new(config: RssConfig) -> Self {
        let scheduler = match JobScheduler::new().await {
            Ok(result) => Arc::new(result),
            Err(e) => {
                panic!("meh")
            }
        };

        Self { config, scheduler }
    }
}
