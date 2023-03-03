mod commands;
mod config;
mod nostr;
mod rss;
mod scheduler;

use std::{collections::HashMap, io, sync::Arc};

use crate::commands::CommandsHandler;
use crate::{rss::rss::RssInstance, scheduler::scheduler::schedule};
use config::AppConfig;
use dotenv::dotenv;
use log::info;
use nostr::nostr::NostrInstance;
use nostr_sdk::Result;
use tokio::{sync::Mutex, task};

#[derive(Clone)]
pub struct App {
    nostr: NostrInstance,
    rss: RssInstance,
    config: AppConfig,
    memory_map: HashMap<String, String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env configuration
    dotenv().ok();

    // init env logger
    env_logger::init();

    // Load app configs
    let app_config = AppConfig::new();

    // Start services
    let nostr_instance = NostrInstance::new(app_config.nostr_config.clone()).await;
    info!("Nostr instance successfully started");

    let rss_instance = RssInstance::new(app_config.rss_config.clone()).await;
    info!("Rss Feed instance successfully started");

    let feeds = rss_instance.config.feeds.clone();

    // Build the main app instance.
    let app = App {
        nostr: nostr_instance.clone(),
        rss: rss_instance,
        config: app_config,
        memory_map: HashMap::new(),
    };

    // Declare identity on Nostr
    let config = &app.nostr.config;
    let _ = &app.nostr.update_profile(config).await.unwrap();

    // Arc wrapper to share nostr instance beetween jobs
    let shared_nostr = Arc::new(Mutex::new(app.nostr));
    let main_data_arc = Arc::new(Mutex::new(app.memory_map));

    /*
    Start cronjob for each feed.
    */
    for feed in feeds {
        let shared_data_arc = Arc::clone(&main_data_arc);
        let client = Arc::clone(&shared_nostr);

        let f = feed.clone();
        let scheduler_rule = f.schedule.as_str();

        let job = schedule(scheduler_rule, feed, shared_data_arc, client).await;
        info!("Job id for feed {:?}: {:?}", f.name, job.guid());

        let _ = &app.rss.scheduler.add(job).await;
    }

    // Start jobs
    let _ = &app.rss.scheduler.start().await;

    let shared_rss = Arc::new(Mutex::new(app.rss));

    // Input handler
    let _ = CommandsHandler::new(shared_rss, shared_nostr, main_data_arc);

    // Loop to maintain program running
    loop {}
}
