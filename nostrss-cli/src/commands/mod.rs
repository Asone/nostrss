use crate::Commands;

pub trait CommandsHandler {
    fn handle(command: Commands) -> String;
}

pub mod feed;
pub mod profile;
pub mod relay;
