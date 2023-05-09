use serde::{Deserialize, Serialize};
use std::env;
use std::net::SocketAddr;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Relay {
    pub name: String,
    pub target: String,
    pub active: bool,
    pub proxy: Option<SocketAddr>,
    #[serde(default = "Relay::default_pow_level")]
    pub pow_level: u8,
}

// into() implementation. Will return the URL string
// representation of the relay.
impl Into<String> for Relay {
    fn into(self) -> String {
        self.target
    }
}

impl Relay {
    fn default_pow_level() -> u8 {
        match env::var("DEFAULT_POW_LEVEL")
            .unwrap_or("0".to_string())
            .parse::<u8>()
        {
            Ok(result) => result,
            Err(_) => 0,
        }
    }
}
