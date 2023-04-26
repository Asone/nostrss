#![allow(dead_code)]

use super::CommandsHandler;

pub struct RelayCommands {}

impl CommandsHandler for RelayCommands {
    fn handle(_command: crate::Commands) -> String {
        "OP_RELAY".to_string()
    }
}

impl RelayCommands {

    fn add() {}

    fn remove() {}
}
