#![allow(dead_code)]

use super::CommandsHandler;
use clap::{Parser, ValueEnum};
use nostrss_grpc::grpc::nostrss_grpc_client::NostrssGrpcClient;
use tonic::async_trait;
use tonic::transport::Channel;

#[derive(Clone, PartialEq, Parser, Debug, ValueEnum)]
pub enum RelayActions {
    Add,
    Delete,
    List,
}

pub struct RelayCommandsHandler {
    pub client: NostrssGrpcClient<Channel>,
}

#[async_trait]
impl CommandsHandler for RelayCommandsHandler {}

impl RelayCommandsHandler {
    pub async fn handle(&mut self, action: RelayActions) {
        match action {
            RelayActions::Add => self.add(),
            RelayActions::Delete => self.delete(),
            RelayActions::List => self.list(),
        }
    }
    fn list(&self) {
        println!("List relays");
    }

    fn add(&self) {
        println!("add relay");
    }

    fn delete(&mut self) {
        println!("add relay");
    }
}
