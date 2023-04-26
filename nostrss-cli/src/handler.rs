use log::error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

use crate::{Commands};

pub struct CliHandler {}

impl CliHandler {
    // Sends message to unix socket
    pub async fn send(stream: &mut UnixStream, data: String) {
        _ = stream.write(data.as_bytes()).await;
        _ = stream.shutdown().await;
    }

    // Listens for message from unix socket
    pub async fn response(stream: &mut UnixStream) -> String {
        let mut response = String::new();
        _ = stream.read_to_string(&mut response).await;

        response
    }

    fn feed_dispatcher(mut op_code: String, action: String) -> String {
        _ = match action.as_str() {
            "add" => {
                // Case logic should come here
                op_code = format!("{}ADD", op_code);
            }
            "delete" => {
                // Case logic should come here
                op_code = format!("{}DEL", op_code);
            }
            "list" => {
                // Case logic should come here
                op_code = format!("{}LS", op_code);
            }
            _ => {
                error!("Unknown action")
            }
        };

        op_code
    }

    fn relay_dispatcher(mut op_code: String, action: String) -> String {
        _ = match action.as_str() {
            "add" => {
                // Case logic should come here
                op_code = format!("{}ADD", op_code);
            }
            "delete" => {
                // Case logic should come here
                op_code = format!("{}DEL", op_code);
            }
            "list" => {
                // Case logic should come here
                op_code = format!("{}LS", op_code);
            }
            _ => {
                error!("Unknown action")
            }
        };

        op_code
    }

    fn profile_dispatcher(mut op_code: String, action: String) -> String {
        _ = match action.as_str() {
            "add" => {
                // Case logic should come here
                op_code = format!("{}ADD", op_code);
            }
            "delete" => {
                // Case logic should come here
                op_code = format!("{}DEL", op_code);
            }
            "list" => {
                // Case logic should come here
                op_code = format!("{}LS", op_code);
            }
            _ => {
                error!("Unknown action")
            }
        };

        op_code
    }

    // Dispatches commands through user input workflow
    pub fn dispatcher(command: Commands) -> String {
        let response = match command {
            Commands::State => "S".to_string(),
            Commands::Feed { action } => {
                let prefix_code = "F_".to_string();
                let op_code = Self::feed_dispatcher(prefix_code, action);
                op_code.to_string()
            },
            Commands::Relay { action } => {
                let prefix_code = "R_".to_string();
                let op_code = Self::relay_dispatcher(prefix_code, action);
                op_code.to_string()
            },
            Commands::Profile { action } => {
                let prefix_code = "P_".to_string();
                let op_code = Self::profile_dispatcher(prefix_code, action);
                op_code.to_string()
            },
            // _ => "OP_UNKNOWN".to_string(),
        };

        response
    }
}

#[cfg(test)]
mod tests {
    use tokio::net::{UnixStream, UnixListener};
    use super::*;

    async fn generate_socket() -> UnixStream {

        let socket_path = ".nostrss-test-socket.sock";
        if std::fs::metadata(socket_path).is_ok() {
            _ = std::fs::remove_file(socket_path);
        }

        _ = match UnixListener::bind(socket_path) {
            Ok(stream) => stream,
            Err(e) => {
                error!("{:?}", e);
                panic!("Couldn't start stream")
            }
        };

        UnixStream::connect(socket_path).await.unwrap()
    }


    // To refactor
    #[tokio::test]
    async fn test_send() {
        
        let mut stream = generate_socket().await;

        let r = CliHandler::send(&mut stream,"bla".to_string()).await;

        assert_eq!(r,())
        
    }

    #[tokio::test]
    async fn test_response() {}

    #[test]
    fn test_dispatcher() {
        let relay_add = CliHandler::dispatcher(Commands::Relay { action: "add".to_string() });
        assert_eq!(relay_add,"R_ADD");

        let relay_delete = CliHandler::dispatcher(Commands::Relay { action: "delete".to_string() });
        assert_eq!(relay_delete,"R_DEL");

        let relay_list = CliHandler::dispatcher(Commands::Relay { action: "list".to_string() });
        assert_eq!(relay_list,"R_LS");

        let profile_add = CliHandler::dispatcher(Commands::Profile { action: "add".to_string() });
        assert_eq!(profile_add,"P_ADD");

        let profile_delete = CliHandler::dispatcher(Commands::Profile { action: "delete".to_string() });
        assert_eq!(profile_delete,"P_DEL");

        let profile_list = CliHandler::dispatcher(Commands::Profile { action: "list".to_string() });
        assert_eq!(profile_list,"P_LS");

        let feed_add = CliHandler::dispatcher(Commands::Feed { action: "add".to_string() });
        assert_eq!(feed_add,"F_ADD");

        let feed_delete = CliHandler::dispatcher(Commands::Feed { action: "delete".to_string() });
        assert_eq!(feed_delete,"F_DEL");

        let feed_list = CliHandler::dispatcher(Commands::Feed { action: "list".to_string() });
        assert_eq!(feed_list,"F_LS");
    }
    
    #[test]
    fn test_relay_dispatcher() {

        let mut op_code = "R_".to_string();
        op_code = CliHandler::relay_dispatcher(op_code.clone(), "add".to_string());
        
        assert_eq!(op_code,"R_ADD".to_string());

        let mut op_code = "R_".to_string();
        op_code = CliHandler::relay_dispatcher(op_code.clone(), "delete".to_string());
        
        assert_eq!(op_code,"R_DEL".to_string());

        let mut op_code = "R_".to_string();
        op_code = CliHandler::relay_dispatcher(op_code.clone(), "list".to_string());
        
        assert_eq!(op_code,"R_LS".to_string());
    }

    #[test]
    fn test_profile_dispatcher() {

        let mut op_code = "P_".to_string();
        op_code = CliHandler::profile_dispatcher(op_code.clone(), "add".to_string());
        
        assert_eq!(op_code,"P_ADD".to_string());

        let mut op_code = "P_".to_string();
        op_code = CliHandler::profile_dispatcher(op_code.clone(), "delete".to_string());
        
        assert_eq!(op_code,"P_DEL".to_string());

        let mut op_code = "P_".to_string();
        op_code = CliHandler::profile_dispatcher(op_code.clone(), "list".to_string());
        
        assert_eq!(op_code,"P_LS".to_string());
    }

    #[test]
    fn test_feed_dispatcher() {

        let mut op_code = "F_".to_string();
        op_code = CliHandler::feed_dispatcher(op_code.clone(), "add".to_string());
        
        assert_eq!(op_code,"F_ADD".to_string());

        let mut op_code = "F_".to_string();
        op_code = CliHandler::feed_dispatcher(op_code.clone(), "delete".to_string());
        
        assert_eq!(op_code,"F_DEL".to_string());

        let mut op_code = "F_".to_string();
        op_code = CliHandler::feed_dispatcher(op_code.clone(), "list".to_string());
        
        assert_eq!(op_code,"F_LS".to_string());
    }


}
