#![allow(dead_code)]

use super::CommandsHandler;

pub struct ProfileCommands {}

impl CommandsHandler for ProfileCommands {
    fn handle(_command: crate::Commands) -> String {
        "OP_PROFILE".to_string()
    }
}

impl ProfileCommands {
    fn list(&self) {}

    fn add() {}

    fn remove() {}
}
