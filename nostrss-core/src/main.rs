// mod commands;
mod app;
mod grpc;
mod nostr;
mod profiles;
mod rss;
mod scheduler;
mod template;
use crate::app::app::{App, AppConfig};
use crate::scheduler::scheduler::schedule;
use clap::Parser;
use dotenv::dotenv;
use grpc::grpc_service::NostrssServerService;
use log::info;
use nostr_sdk::Result;
use nostrss_grpc::grpc::nostrss_grpc_server::NostrssGrpcServer;
use std::env;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use tonic::transport::Server;

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

    // Create app instance
    let app = App::new(AppConfig::parse()).await;

    // Extract initial feeds list
    let feeds = app.rss.feeds.clone();

    // Arc the main app
    let global_app_arc = Arc::new(Mutex::new(app));

    // Update profile for each client
    _ = {
        let global_app_lock = global_app_arc.lock().await;
        for client in global_app_lock.clients.clone().into_iter() {
            let result = client.1.update_profile().await;

            println!("result of profile update for {} : {:?}", client.0, result);
        }
    };

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

    // GRPC server
    _ = {
        let local_app = Arc::clone(&global_app_arc);

        let grpc_address = env::var("GRPC_ADDRESS").unwrap_or("[::1]:33333".to_string());
        let address = grpc_address.parse().unwrap();

        let nostrss_grpc = NostrssServerService { app: local_app };

        match Server::builder()
            .add_service(NostrssGrpcServer::new(nostrss_grpc))
            .serve(address)
            .await
        {
            Ok(r) => println!("{:?}", r),
            Err(e) => panic!("{:?}", e),
        };
    };

    // Loop to maintain program running
    loop {
        // Sleep to avoid useless high CPU usage
        sleep(Duration::from_millis(100));
    }
}
