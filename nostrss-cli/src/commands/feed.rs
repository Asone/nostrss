#![allow(dead_code)]

use super::CommandsHandler;

pub struct FeedCommands {}

impl CommandsHandler for FeedCommands {
    fn handle(_command: crate::Commands) -> String {
        "OP_FEED".to_string()
    }
}

impl FeedCommands {
    fn list(&self) {}

    fn add() {}

    fn remove() {}
}
