use crate::{
    commands::{
        feed::FeedCommandsHandler, profile::ProfileCommandsHandler, relay::RelayCommandsHandler,
    },
    CliOptions, Subcommands,
};

use nostrss_grpc::grpc::{nostrss_grpc_client::NostrssGrpcClient, StateRequest};
use tonic::transport::Channel;

/// Global handler for the CLI commands.
/// It provides a dispatcher that will send the command
/// details to sub-handlers
pub struct CliHandler {
    pub client: NostrssGrpcClient<Channel>,
}

impl CliHandler {
    pub async fn dispatcher(&mut self, command: Subcommands, opts: CliOptions) {
        match command {
            Subcommands::State => {
                let request = tonic::Request::new(StateRequest {});
                let response = self.client.state(request).await;
                match response {
                    Ok(r) => {
                        println!("{}", r.into_inner().state);
                    }
                    Err(e) => {
                        println!("error: {}", e);
                    }
                }
            }
            Subcommands::Feed { action } => {
                let mut feed_handler = FeedCommandsHandler {
                    client: self.client.clone(),
                };
                feed_handler.handle(action, opts).await;
            }
            Subcommands::Relay { action } => {
                let mut relay_handler = RelayCommandsHandler {
                    client: self.client.clone(),
                };
                println!("Relay commands");
                relay_handler.handle(action).await;
            }
            Subcommands::Profile { action } => {
                let mut profile_handler = ProfileCommandsHandler {
                    client: self.client.clone(),
                };
                profile_handler.handle(action, opts).await;
            }
        };
    }
}

#[cfg(test)]
mod tests {}
