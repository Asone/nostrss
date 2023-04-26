use std::{io::Error, sync::Arc};

use log::{error, info};
use regex::Regex;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{UnixListener, UnixStream},
    sync::Mutex,
};

use crate::{
    app::app::App,
    socket::{
        feed::FeedCommandHandler, profile::ProfileCommandHandler, relay::RelayCommandHandler,
    },
};

/// This struct provides the socket processing management.
pub struct SocketHandler {
    pub stream: UnixListener,
}

impl SocketHandler {
    // Creates a new instance of struct
    pub fn new(socket_path: &str) -> Self {
        if std::fs::metadata(socket_path).is_ok() {
            info!("A socket is already present. Deleting...");
            _ = std::fs::remove_file(socket_path);
        }

        let stream = match UnixListener::bind(socket_path) {
            Ok(stream) => stream,
            Err(e) => {
                error!("{:?}", e);
                panic!("Couldn't start stream")
            }
        };
        Self { stream }
    }

    pub async fn listen(&self, app: Arc<Mutex<App>>) -> Result<String, Error> {
        let s = self.stream.accept().await;

        let mut op_code = String::new();

        match s {
            Ok((mut stream, _addr)) => {
                _ = stream.read_to_string(&mut op_code).await;

                self.dispatch(stream, app, op_code).await;
                Ok("".to_string())
            }
            Err(e) => {
                error!("Connection failed");

                Err(e)
            }
        }
    }

    // Dispatches commands through user workflow
    pub async fn dispatch(&self, mut stream: UnixStream, app: Arc<Mutex<App>>, op_code: String) {
        let cat_regex = Regex::new(r"^([A-Z])_([A-Z]{2,4})");

        if cat_regex.is_err() {
            error!("Could not parse command");
            ()
        }

        let category = cat_regex.unwrap().captures(op_code.as_str());
        println!("{:?}", category);

        if category.is_none() {
            "Unknown command".to_string();
        };

        let category = category.unwrap();

        // action
        let action = match &category.get(2) {
            Some(a) => a.as_str(),
            None => "",
        };

        // category
        let category = match &category.get(1) {
            Some(c) => c.as_str(),
            None => "",
        };

        let response = match category {
            "R" => RelayCommandHandler::handle(app, action.to_string()).await,
            "P" => ProfileCommandHandler::handle(app, action.to_string()).await,
            "F" => FeedCommandHandler::handle(app, action.to_string()).await,
            _ => "Unknown command".to_string(),
        };
        _ = stream.write(response.as_bytes()).await;
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_dispatch_method() {}

    #[test]
    fn test_new_method() {}

    #[test]
    fn test_listen_method() {}
}
