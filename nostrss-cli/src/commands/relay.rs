#![allow(dead_code)]
use crate::RelayActions;

use super::CommandsHandler;
use nostrss_grpc::grpc::nostrss_grpc_client::NostrssGrpcClient;
use tonic::async_trait;
use tonic::transport::Channel;

pub struct RelayCommandsHandler {
    pub client: NostrssGrpcClient<Channel>,
}

#[async_trait]
impl CommandsHandler for RelayCommandsHandler {
    async fn handle(&mut self, action: String) {
        match action.as_str() {
            "add" => self.add(),
            "delete" => self.delete(),
            "list" => self.list(),
            _ => {}
        }
    }
}

impl RelayCommandsHandler {
    fn list(&self) {}

    fn add(&self) {}

    fn delete(&mut self) {}
}
