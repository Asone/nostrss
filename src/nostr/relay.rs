use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Relay {
    pub name: String,
    pub target: String,
    pub active: bool,
    pub proxy: Option<SocketAddr>,
}

// into() implementation. Will return the URL string
// representation of the relay.
impl Into<String> for Relay {
    fn into(self) -> String {
        self.target
    }
}
