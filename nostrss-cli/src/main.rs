mod commands;
/// Nostrss-cli provides a bridge beetween rss feeds and [Nostr protocol](https://nostr.com/).
///
/// To use it, you will have to provide some configuration, like relays and feeds to load, which are described into
/// the [README.md](https://github.com/Asone/nostrss/blob/main/README.md) file..
//: The application is based on async cronjobs.
mod handler;
mod input;
use std::{env, process::exit};

use commands::{feed::FeedActions, profile::ProfileActions, relay::RelayActions};
use dotenv::dotenv;
use handler::CliHandler;

use clap::{command, Parser};

use nostrss_grpc::grpc::nostrss_grpc_client::NostrssGrpcClient;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    subcommand: Subcommands,
    #[arg(long, short)]
    /// Save the modifications to config files
    pub save: bool,
}

#[derive(Debug, Default)]
pub struct CliOptions {
    save: bool,
}

#[derive(Debug, PartialEq, Parser)]
pub enum Subcommands {
    #[clap(
        name = "relay",
        about = "Provides commands for relay management",
        long_about = r#"
            Available actions for relays: 
            * add : Add a relay
            * delete : Removes a relay
            * list : List the active relays
            * info : Get info on a relay
"#
    )]
    Relay {
        #[clap(name = "action")]
        action: RelayActions,
    },

    #[clap(name = "feed", about = "Provides commands for feed mcanagement")]
    Feed { action: FeedActions },
    /// Provides commands for Profile management
    Profile { action: ProfileActions },
    /// Checks health of core
    State,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let grpc_address = env::var("GRPC_ADDRESS").unwrap_or("[::1]:33333".to_string());

    let grpc_full_address = format!("{}{}", "http://", grpc_address);
    // Creates the gRPC client
    let client = match NostrssGrpcClient::connect(grpc_full_address).await {
        Ok(c) => c,
        Err(e) => {
            log::error!("Could not connect to core service. Are you sure it is up ?");
            panic!("{}", e);
        }
    };

    // Get CLI arguments and parameters
    let cli = Cli::parse();

    let opts = CliOptions {
        save: cli.save,
        ..Default::default()
    };

    let mut handler = CliHandler { client };
    handler.dispatcher(cli.subcommand, opts).await;

    // If we reach this point we close the program gracefully
    exit(1);
}
