use std::{collections::HashMap, io, str::SplitWhitespace, sync::Arc};

use log::{info, warn};
use tokio::sync::Mutex;

use crate::{nostr::nostr::NostrInstance, rss::rss::RssInstance};
use tokio::task;

/// Handles input commands
pub struct CommandsHandler {}

impl CommandsHandler {
    pub fn new(
        rss: Arc<Mutex<RssInstance>>,
        nostr: Arc<Mutex<NostrInstance>>,
        map: Arc<Mutex<HashMap<String, Vec<String>>>>,
    ) -> task::JoinHandle<()> {
        task::spawn(async move {
            let mut input = String::new();

            let client = Arc::clone(&nostr);
            let rss = Arc::clone(&rss);
            let _map_arc = Arc::clone(&map);

            loop {
                // Read input from stdin
                io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read input");

                // Send input content to dispatcher
                Self::dispatch(&input, &client, &rss).await;

                // Clear input once processed
                input.clear();
            }
        })
    }

    // Dispatches an input to sub-handlers
    pub async fn dispatch(
        input: &str,
        nostr: &Arc<Mutex<NostrInstance>>,
        rss: &Arc<Mutex<RssInstance>>,
    ) -> () {
        let mut input = input.split_whitespace();
        let arg1 = input.next();

        match arg1 {
            Some(command) => match command {
                "relays" => {
                    Self::relays_handler(input, nostr).await;
                }
                "feeds" => {
                    Self::feeds_handler(input, rss).await;
                }
                _ => warn!("No command handler found for command {:?}", command),
            },
            None => {}
        };
    }

    // handles relays related commands
    async fn relays_handler(
        mut input: SplitWhitespace<'_>,
        nostr_arc: &Arc<Mutex<NostrInstance>>,
    ) -> () {
        let subcommand = input.next();
        match subcommand {
            Some(subcommand) => match subcommand {
                "list" => {
                    let nostr_lock = nostr_arc.lock().await;
                    let relays = nostr_lock.client.relays().await;
                    println!("\n\rRelays list :\n\r");
                    for (key, _value) in relays {
                        println!("Relay URL : {:?}", key.host_str().unwrap_or(""));
                    }
                    println!("\n\r");
                }
                "add" => {
                    let url = input.next();
                    match url {
                        Some(_) => {
                            info!("This command is not implemented yet.")
                        }
                        None => {}
                    }
                }
                "remove" => {}
                _ => {
                    warn!("No command handler found for sub-command {:?}", subcommand);
                    Self::relays_help_message();
                }
            },
            None => {
                println!("Missing sub-command for relays command.");
            }
        }
    }

    // handles feed jobs related commands
    async fn feeds_handler(mut input: SplitWhitespace<'_>, rss: &Arc<Mutex<RssInstance>>) -> () {
        let subcommand = input.next();
        match subcommand {
            Some(subcommand) => match subcommand {
                "list" => {
                    let rss_lock = rss.lock().await;
                    for feed_job in &rss_lock.feeds_jobs {
                        println!("feed id : {:?}, job uuid: {:?}", feed_job.0, feed_job.1);
                    }
                }
                "add" => {}
                "remove" => {
                    let feed_id = input.next();
                    match feed_id {
                        Some(id) => {
                            let rss_lock = &mut rss.lock().await;
                            let feed_id = id.to_string();
                            let feeds_jobs = &rss_lock.feeds_jobs;
                            match &rss_lock.feeds_jobs.contains_key(&feed_id) {
                                true => {
                                    let job_uuid = feeds_jobs[&feed_id];
                                    println!("{:?}", job_uuid);
                                    let _ = &rss_lock.scheduler.remove(&job_uuid).await;
                                    let _ = rss_lock.feeds_jobs.remove_entry(&feed_id);
                                    println!(
                                        "Job for {} feed with uuid {} removed",
                                        &feed_id, &job_uuid
                                    );
                                }
                                false => {
                                    println!("No job matching with this feed id.")
                                }
                            }
                        }
                        None => {
                            println!("No feed id provided.");
                        }
                    }
                }
                _ => {
                    warn!("No command handler found for sub-command {}", subcommand);
                }
            },
            None => {
                warn!("Missing sub-command for feeds command.");
                Self::feeds_help_message();
            }
        }
    }

    fn relays_help_message() -> () {
        let msg = "\n\
        Relays commands:\n\
        \n\
        list : prints the current list of connected relays to app's client\n\
        add : add a new relay to the list\n\
        remove : remove a relay from the list\n\
        \n\
        ";
        print!("{}", msg);
    }

    fn feeds_help_message() -> () {
        let msg = "\n\
        feeds commands:\n\
        \n\
        list : prints the current list of feeds to app's client\n\
        add : add a new feed to the list\n\
        remove : remove a feed from the list\n\
        \n\
        ";
        print!("{}", msg);
    }
}
