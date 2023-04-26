// mod commands;
mod app;
mod nostr;
mod profiles;
mod rss;
mod scheduler;
mod socket;
mod template;
use crate::app::app::{App, AppConfig};
use crate::scheduler::scheduler::schedule;
use clap::Parser;
use dotenv::dotenv;
use log::info;
use nostr_sdk::Result;
use socket::handler::SocketHandler;
use std::sync::Arc;

use tokio::sync::Mutex;

/// Nostrss provides a bridge beetween rss feeds and [Nostr protocol](https://nostr.com/).
///
/// To use it, you will have to provide some configuration, like relays and feeds to load, which are described into
/// the [README.md](https://github.com/Asone/nostrss/blob/main/README.md) file..
//: The application is based on async cronjobs.
#[tokio::main]
async fn main() -> Result<()> {
    // Load .env configuration
    dotenv().ok();

    // init env logger
    env_logger::init();

    // Create Unix socket for CLI util
    let socket_path = ".nostrss-socket.sock";
    let socket_handler = Arc::new(Mutex::new(SocketHandler::new(socket_path)));

    // Create app instance
    let app = App::new(AppConfig::parse()).await;

    // // Extract initial feeds list
    let feeds = app.rss.feeds.clone();

    // // Arc the main app
    let global_app_arc = Arc::new(Mutex::new(app));

    /*
    Build job for each feed.
    */
    for feed in feeds {
        // Lock the app mutex
        let mut app_lock = global_app_arc.lock().await;

        // Local instance of feed
        let f = feed.clone();

        // Arc and lock the clients to extract the associated client
        // for the feed Based on the profile id.
        let clients_arc = Arc::new(Mutex::new(app_lock.clients.clone()));

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

    // Start jobs.
    // We scope the instructions in a block to avoidd
    // locking the app arc on the whole instance as we
    // need to be able to lock it again later.
    _ = {
        let app_lock = global_app_arc.lock().await;
        _ = &app_lock.rss.scheduler.start().await;
    };

    // Loop to maintain program running
    loop {
        // Sleep to avoid useless high CPU usage
        // sleep(Duration::from_millis(100));
        let local_app = Arc::clone(&global_app_arc);

        // Socket handler
        let stream_lock = socket_handler.lock().await;
        _ = stream_lock.listen(local_app).await;
    }
}
