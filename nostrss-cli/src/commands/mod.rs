use std::io::{self, stdin, Write};

use tonic::async_trait;

/// Common trait for sub-handlers.
#[async_trait]
pub trait CommandsHandler {
    // All sub-handlers should have a unique entrypoint
    // to dispatch the actions.
    async fn handle(&mut self, command: String);

    // A default helper to get input data from user.
    fn get_input(&self, label: &str) -> String {
        let mut data = String::new();
        print!("{}", label);
        _ = io::stdout().flush();
        _ = stdin().read_line(&mut data);
        data
    }
}

pub mod feed;
pub mod profile;
pub mod relay;
