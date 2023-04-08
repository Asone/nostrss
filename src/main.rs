// mod commands;
mod app;
mod nostr;
mod profiles;
mod rss;
mod scheduler;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use crate::app::app::{App, AppConfig};
use crate::scheduler::scheduler::schedule;
use clap::Parser;
use dotenv::dotenv;
use log::info;
use nostr_sdk::Result;
use tokio::sync::Mutex;

/// Nostrss provides a bridge beetween rss feeds and [Nostr protocol](https://nostr.com/).
///
/// To use it, you will have to provide some configuration, like relays and feeds to load, which are described into
/// the [README.md](https://github.com/Asone/nostrss/blob/main/README.md) file..
#[tokio::main]
async fn main() -> Result<()> {
    // Load .env configuration
    dotenv().ok();

    // init env logger
    env_logger::init();

    // Create app instance
    let app = App::new(AppConfig::parse()).await;

    // Arc the main app
    let global_app_arc = Arc::new(Mutex::new(app));

    // Lock the app mutex
    let mut app_lock = global_app_arc.lock().await;

    // Extract initial feeds list
    let feeds = app_lock.rss.feeds.clone();

    /*
    Build job for each feed.
    */
    for feed in feeds {
        // Local instance of feed
        let f = feed.clone();

        // Arc and lock the clients to extract the associated client
        // for the feed Based on the profile id.
        let clients_arc = Arc::new(Mutex::new(app_lock.clients.clone()));
        // let clients_lock = clients_arc.lock().await;

        // let client = Arc::new(Mutex::new(clients_lock[profile_id].clone()));

        // Arc the map of feeds for use in the scheduled jobs
        let maps = Arc::new(Mutex::new(app_lock.feeds_map.clone()));

        // Extract cronjob rule
        let scheduler_rule = f.schedule.as_str();

        // Call job builder
        let job = schedule(scheduler_rule, feed, maps, clients_arc).await;
        info!("Job id for feed {:?}: {:?}", f.name, job.guid());

        // Load job reference in jobs map
        _ = &app_lock.rss.feeds_jobs.insert(f.id, job.guid());

        // Load job in scheduler
        _ = &app_lock.rss.scheduler.add(job).await;
    }

    // Start jobs
    let _ = &app_lock.rss.scheduler.start().await;

    // Input handler
    // _ = CommandsHandler::new(shared_rss, shared_nostr, feed_fingerprints);

    // Loop to maintain program running
    loop {
        // Sleep to avoid useless high CPU usage
        sleep(Duration::from_millis(100));
    }
}
