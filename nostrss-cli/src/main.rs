mod commands;
/// Nostrss-cli provides a bridge beetween rss feeds and [Nostr protocol](https://nostr.com/).
///
/// To use it, you will have to provide some configuration, like relays and feeds to load, which are described into
/// the [README.md](https://github.com/Asone/nostrss/blob/main/README.md) file..
//: The application is based on async cronjobs.
mod handler;

use dotenv::dotenv;
use handler::CliHandler;
use tokio::{
    net::{UnixStream}
};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Relay {
    Add,
    Delete,
    List,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Provides commands for relay management
    Relay { action: String },
    /// Provides commands for feed management
    Feed { action: String },
    /// Provides commands for Profile management
    Profile { action: String },
    /// Checks health of core
    State,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let socket_path = ".nostrss-socket.sock";

    // Get CLI arguments and parameters
    let cli = Cli::parse();

    // Create unix socket connection
    let unix_stream = UnixStream::connect(socket_path).await;

    // In case we fail connecting
    if unix_stream.is_err() {
        panic!("Could not connect to socket. Exiting...");
    }

    let mut stream = unix_stream.unwrap();

    let result = CliHandler::dispatcher(cli.command);
    _ = CliHandler::send(&mut stream, result).await;
    let response = CliHandler::response(&mut stream).await;

    println!("{}", response);
}