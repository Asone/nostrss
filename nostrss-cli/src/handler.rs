use crate::{
    commands::{
        feed::FeedCommandsHandler, profile::ProfileCommandsHandler, relay::RelayCommandsHandler,
        CommandsHandler,
    },
    Subcommands,
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
    pub async fn dispatcher(&mut self, command: Subcommands) {
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
                feed_handler.handle(action).await;
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
                profile_handler.handle(action).await;
            }
        };
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use tokio::net::{UnixListener, UnixStream};

//     async fn generate_socket() -> UnixStream {
//         let socket_path = ".nostrss-test-socket.sock";
//         if std::fs::metadata(socket_path).is_ok() {
//             _ = std::fs::remove_file(socket_path);
//         }

//         _ = match UnixListener::bind(socket_path) {
//             Ok(stream) => stream,
//             Err(e) => {
//                 error!("{:?}", e);
//                 panic!("Couldn't start stream")
//             }
//         };

//         UnixStream::connect(socket_path).await.unwrap()
//     }

//     // To refactor
//     // #[tokio::test]
//     // async fn test_send() {

//     //     let mut stream = generate_socket().await;

//     //     let r = CliHandler::send(&mut stream,"bla".to_string()).await;

//     //     assert_eq!(r,())

//     // }

//     #[tokio::test]
//     async fn test_response() {}

//     #[test]
//     fn test_dispatcher() {
//         let relay_add = CliHandler::dispatcher(Commands::Relay {
//             action: "add".to_string(),
//         });
//         assert_eq!(relay_add, "R_ADD");

//         let relay_delete = CliHandler::dispatcher(Commands::Relay {
//             action: "delete".to_string(),
//         });
//         assert_eq!(relay_delete, "R_DEL");

//         let relay_list = CliHandler::dispatcher(Commands::Relay {
//             action: "list".to_string(),
//         });
//         assert_eq!(relay_list, "R_LS");

//         let profile_add = CliHandler::dispatcher(Commands::Profile {
//             action: "add".to_string(),
//         });
//         assert_eq!(profile_add, "P_ADD");

//         let profile_delete = CliHandler::dispatcher(Commands::Profile {
//             action: "delete".to_string(),
//         });
//         assert_eq!(profile_delete, "P_DEL");

//         let profile_list = CliHandler::dispatcher(Commands::Profile {
//             action: "list".to_string(),
//         });
//         assert_eq!(profile_list, "P_LS");

//         let feed_add = CliHandler::dispatcher(Commands::Feed {
//             action: "add".to_string(),
//         });
//         assert_eq!(feed_add, "F_ADD");

//         let feed_delete = CliHandler::dispatcher(Commands::Feed {
//             action: "delete".to_string(),
//         });
//         assert_eq!(feed_delete, "F_DEL");

//         let feed_list = CliHandler::dispatcher(Commands::Feed {
//             action: "list".to_string(),
//         });
//         assert_eq!(feed_list, "F_LS");
//     }

//     #[test]
//     fn test_relay_dispatcher() {
//         let mut op_code = "R_".to_string();
//         op_code = CliHandler::relay_dispatcher(op_code.clone(), "add".to_string());

//         assert_eq!(op_code, "R_ADD".to_string());

//         let mut op_code = "R_".to_string();
//         op_code = CliHandler::relay_dispatcher(op_code.clone(), "delete".to_string());

//         assert_eq!(op_code, "R_DEL".to_string());

//         let mut op_code = "R_".to_string();
//         op_code = CliHandler::relay_dispatcher(op_code.clone(), "list".to_string());

//         assert_eq!(op_code, "R_LS".to_string());
//     }

//     #[test]
//     fn test_profile_dispatcher() {
//         let mut op_code = "P_".to_string();
//         op_code = CliHandler::profile_dispatcher(op_code.clone(), "add".to_string());

//         assert_eq!(op_code, "P_ADD".to_string());

//         let mut op_code = "P_".to_string();
//         op_code = CliHandler::profile_dispatcher(op_code.clone(), "delete".to_string());

//         assert_eq!(op_code, "P_DEL".to_string());

//         let mut op_code = "P_".to_string();
//         op_code = CliHandler::profile_dispatcher(op_code.clone(), "list".to_string());

//         assert_eq!(op_code, "P_LS".to_string());
//     }

//     #[test]
//     fn test_feed_dispatcher() {
//         let mut op_code = "F_".to_string();
//         op_code = CliHandler::feed_dispatcher(op_code.clone(), "add".to_string());

//         assert_eq!(op_code, "F_ADD".to_string());

//         let mut op_code = "F_".to_string();
//         op_code = CliHandler::feed_dispatcher(op_code.clone(), "delete".to_string());

//         assert_eq!(op_code, "F_DEL".to_string());

//         let mut op_code = "F_".to_string();
//         op_code = CliHandler::feed_dispatcher(op_code.clone(), "list".to_string());

//         assert_eq!(op_code, "F_LS".to_string());
//     }
// }
