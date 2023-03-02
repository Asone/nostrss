use std::{str::SplitWhitespace, sync::Arc};

use log::warn;
use tokio::sync::Mutex;

use crate::{nostr::nostr::NostrInstance, rss::rss::RssInstance};

pub struct CommandsHandler {}

impl CommandsHandler {
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
                    println!("Relays list :");
                    for (key, _value) in relays {
                        println!("Relay URL : {:?}", key.host_str().unwrap_or(""));
                    }
                    println!("############################");
                }
                "add" => {
                    let url = input.next();
                    match url {
                        Some(url) => {}
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

    fn feeds_handler(mut input: SplitWhitespace, rss: &Arc<Mutex<RssInstance>>) -> () {
        let subcommand = input.next();
        match subcommand {
            Some(subcommand) => match subcommand {
                "list" => {}
                "add" => {}
                "remove" => {}
                _ => {
                    warn!("No command handler found for sub-command {:?}", subcommand);
                }
            },
            None => {
                warn!("Missing sub-command for feeds command.");
                Self::feeds_help_message();
            }
        }
    }

    fn relays_help_message() -> () {
        let msg = "Relays commands:\n\
        list : prints the current list of connected relays to app's client\n\
        add : add a new relay to the list\n\
        remove : remove a relay from the list
        ";
        print!("{}", msg);
    }
    fn feeds_help_message() -> () {
        let msg = "feeds commands:\n\
        list : prints the current list of feeds to app's client\n\
        add : add a new feed to the list\n\
        remove : remove a feed from the list
        ";
        print!("{}", msg);
    }

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
                    Self::feeds_handler(input, rss);
                }
                _ => warn!("No command handler found for command {:?}", command),
            },
            None => {}
        };
    }
}
