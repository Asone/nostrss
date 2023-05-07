mod commands;
/// Nostrss-cli provides a bridge beetween rss feeds and [Nostr protocol](https://nostr.com/).
///
/// To use it, you will have to provide some configuration, like relays and feeds to load, which are described into
/// the [README.md](https://github.com/Asone/nostrss/blob/main/README.md) file..
//: The application is based on async cronjobs.
mod handler;

use std::process::exit;

use dotenv::dotenv;
use handler::CliHandler;

use clap::{command, Parser, Subcommand, ValueEnum};

use nostrss_grpc::grpc::nostrss_grpc_client::NostrssGrpcClient;
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    subcommand: Subcommands,
}

#[derive(Clone, PartialEq, Parser, Debug, ValueEnum)]
pub enum RelayActions {
    #[clap(name = "add")]
    Add,
    Delete,
    List,
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
        #[clap(name = "action", long = "The action to be done")]
        action: String,
    },

    #[clap(name = "feed", about = "Provides commands for feed mcanagement")]
    Feed { action: String },
    /// Provides commands for Profile management
    Profile { action: String },
    /// Checks health of core
    State,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Creates the gRPC client
    let client = match NostrssGrpcClient::connect("http://[::1]:9999").await {
        Ok(c) => c,
        Err(e) => {
            log::error!("Could not connect to core service. Are you sure it is up ?");
            panic!("{}", e);
        }
    };

    // Get CLI arguments and parameters
    let cli = Cli::parse();

    let mut handler = CliHandler { client };
    _ = handler.dispatcher(cli.subcommand).await;

    // If we reach this point we close the program gracefully
    exit(1);
}
