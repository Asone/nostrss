#![allow(dead_code)]

use core::panic;
use log::{info, warn};
use nostr_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use std::{env, net::SocketAddr, path::Path};

use crate::config::Args;

#[derive(Debug)]
pub enum NostrConfigErrors {
    FileLocationError,
    FileFormatError,
    FileParsingError,
    KeyParsingError,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Relay {
    name: String,
    pub target: String,
    active: bool,
    pub proxy: Option<SocketAddr>,
}

impl Into<String> for Relay {
    fn into(self) -> String {
        return self.target;
    }
}

#[derive(Debug, Clone)]
pub struct NostrConfig {
    pub keys: Keys,
    pub relays: Vec<Relay>,
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub about: Option<String>,
    pub description: Option<String>,
    pub picture: Option<String>,
    pub banner: Option<String>,
    pub nip05: Option<String>,
    pub lud16: Option<String>,
}

impl NostrConfig {
    pub fn new(args: &Args) -> Self {
        let keys = match &args.private_key {
            Some(private_keys) => match Self::set_keys(private_keys) {
                Ok(keys) => keys,
                Err(_) => {
                    panic!("{:#?}", NostrConfigErrors::KeyParsingError)
                }
            },
            None => Self::load_keys(),
        };

        info!("public key : {:?}", &keys.public_key());
        info!(
            "bech32 public key : {:?}",
            &keys.public_key().to_bech32().unwrap()
        );
        let mut config = Self {
            keys: keys,
            relays: Vec::new(),
            about: Self::get_env_description(),
            name: Self::get_env_name(),
            display_name: Self::get_env_display_name(),
            description: Self::get_env_description(),
            picture: None,
            banner: None,
            nip05: None,
            lud16: None,
        };

        if args.relays.is_some() {
            info!("Found relays file path argument. Parsing file...");
            config = config.load_relays(&args.relays.as_ref().unwrap());
        }

        config
    }

    pub fn set_relays(mut self, relays: Vec<Relay>) -> Self {
        self.relays = relays;
        self
    }

    pub fn set_display_name(mut self, display_name: Option<String>) -> Self {
        self.display_name = display_name;
        self
    }

    pub fn get_display_name(self) -> Option<String> {
        if self.display_name.is_some() {
            return self.display_name;
        }

        Self::get_env_display_name()
    }

    fn get_env_display_name() -> Option<String> {
        match env::var("NOSTR_DISPLAY_NAME")
            .unwrap_or("".to_string())
            .parse::<String>()
        {
            Ok(result) => match result.len() > 0 {
                true => Some(result),
                false => None,
            },
            Err(_) => None,
        }
    }

    pub fn set_name(mut self, name: Option<String>) -> Self {
        self.name = name;
        self
    }

    pub fn get_name(self) -> Option<String> {
        if self.name.is_some() {
            return self.name;
        }

        Self::get_env_name()
    }

    fn get_env_name() -> Option<String> {
        match env::var("NOSTR_NAME")
            .unwrap_or("".to_string())
            .parse::<String>()
        {
            Ok(result) => match result.len() > 0 {
                true => Some(result),
                false => None,
            },
            Err(_) => None,
        }
    }

    pub fn set_description(mut self, description: Option<String>) -> Self {
        self.description = description;
        self
    }

    pub fn get_description(self) -> Option<String> {
        if self.description.is_some() {
            return self.description;
        }

        Self::get_env_description()
    }

    fn get_env_description() -> Option<String> {
        match env::var("NOSTR_DESCRIPTION")
            .unwrap_or("".to_string())
            .parse::<String>()
        {
            Ok(result) => match result.len() > 0 {
                true => Some(result),
                false => None,
            },
            Err(_) => None,
        }
    }

    pub fn set_picture(mut self, picture: Option<String>) -> Self {
        self.picture = picture;
        self
    }

    pub fn get_picture(self) -> Option<String> {
        if self.picture.is_some() {
            return self.picture;
        }

        Self::get_env_picture()
    }

    fn get_env_picture() -> Option<String> {
        match env::var("NOSTR_PICTURE")
            .unwrap_or("".to_string())
            .parse::<String>()
        {
            Ok(result) => match result.len() > 0 {
                true => Some(result),
                false => None,
            },
            Err(_) => None,
        }
    }

    pub fn set_banner(mut self, banner: Option<String>) -> Self {
        self.banner = banner;
        self
    }

    pub fn get_banner(self) -> Option<String> {
        if self.banner.is_some() {
            return self.banner;
        }

        Self::get_env_banner()
    }

    fn get_env_banner() -> Option<String> {
        match env::var("NOSTR_BANNER")
            .unwrap_or("".to_string())
            .parse::<String>()
        {
            Ok(result) => match result.len() > 0 {
                true => Some(result),
                false => None,
            },
            Err(_) => None,
        }
    }

    pub fn set_nip05(mut self, nip05: Option<String>) -> Self {
        self.nip05 = nip05;
        self
    }

    pub fn get_nip05(self) -> Option<String> {
        if self.nip05.is_some() {
            return self.nip05;
        }

        match env::var("NOSTR_NIP05")
            .unwrap_or("".to_string())
            .parse::<String>()
        {
            Ok(result) => match result.len() > 0 {
                true => Some(result),
                false => None,
            },
            Err(_) => None,
        }
    }

    pub fn get_lud16(self) -> Option<String> {
        if self.lud16.is_some() {
            return self.lud16;
        }

        match env::var("NOSTR_LUD16")
            .unwrap_or("".to_string())
            .parse::<String>()
        {
            Ok(result) => match result.len() > 0 {
                true => Some(result),
                false => None,
            },
            Err(_) => None,
        }
    }

    pub fn set_lud16(mut self, lud16: Option<String>) -> Self {
        self.lud16 = lud16;
        self
    }

    fn get_env_var(var_name: &str) -> Option<String> {
        match env::var(&format!("NOSTR_{}", var_name.to_uppercase())) {
            Ok(val) => Some(val),
            Err(_) => None,
        }
    }

    fn set_keys(secret_key: &String) -> Result<Keys, NostrConfigErrors> {
        match Keys::from_sk_str(secret_key) {
            Ok(keys) => Ok(keys),
            Err(_) => return Err(NostrConfigErrors::KeyParsingError),
        }
    }

    pub fn load_keys() -> Keys {
        match env::var("NOSTR_PK") {
            Ok(val) => match Keys::from_sk_str(&val) {
                Ok(val) => val,
                Err(_) => {
                    warn!("Invalid private key found for Nostr. Generating random keys...");
                    Keys::generate()
                }
            },
            Err(_) => {
                warn!("No private key found for Nostr. Generating random keys...");
                Keys::generate()
            }
        }
    }

    pub fn load_json_relays(mut self, path: &Path) -> Self {
        let file = match std::fs::File::open(path) {
            Ok(file) => file,
            Err(_) => {
                return self;
            }
        };
        let relays: Vec<Relay> = match serde_json::from_reader(file) {
            Ok(relays) => relays,
            Err(_) => return self,
        };

        self.relays = relays;
        self
    }

    pub fn load_relays(self, path: &str) -> Self {
        let path = Path::new(path);

        if path.is_file() {
            match path.extension() {
                Some(ext) => match ext.to_str() {
                    Some("yml") => {
                        return self.load_yaml_relays(path);
                    }
                    Some("yaml") => {
                        return self.load_yaml_relays(path);
                    }
                    Some("json") => {
                        return self.load_json_relays(path);
                    }
                    _ => {
                        return self;
                    }
                },
                None => {
                    return self;
                }
            }
        }

        self
    }

    pub fn load_yaml_relays(mut self, path: &Path) -> Self {
        let file = match std::fs::File::open(path) {
            Ok(file) => file,
            Err(_) => {
                return self;
            }
        };
        let relays: Vec<Relay> = match serde_yaml::from_reader(file) {
            Ok(relays) => relays,
            Err(_) => return self,
        };

        self.relays = relays;
        self
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    // Test the `NostrConfig` constructor with empty arguments
    #[test]
    fn test_nostrconfig_new_empty_args() {
        let args = Args {
            private_key: None,
            relays: None,
            feeds: None,
        };
        let config = NostrConfig::new(&args);
        assert_eq!(config.relays.len(), 0);
        assert_eq!(config.name, None);
        assert_eq!(config.display_name, None);
        assert_eq!(config.about, None);
        assert_eq!(config.description, None);
        assert_eq!(config.picture, None);
        assert_eq!(config.banner, None);
        assert_eq!(config.nip05, None);
        assert_eq!(config.lud16, None);
    }

    // Test the `NostrConfig` constructor with private key argument
    #[test]
    fn test_nostrconfig_new_private_key_arg() {
        let args = Args {
            private_key: Some(String::from(
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            )),
            relays: None,
            feeds: None,
        };
        let config = NostrConfig::new(&args);
        assert_eq!(
            config.keys.public_key().to_bech32().unwrap(),
            "npub1ger2u5z8x945yvxsppkg4nkxslcqk8xe68wxxnmvkdv2cz563lls9fwehy"
        );
    }

    // Test the `NostrConfig` constructor with relays file path argument
    #[test]
    fn test_nostrconfig_new_relays_arg() {
        let relays_file_path = "./src/fixtures/relays.json".to_string();
        let feeds_file_path = "./src/fixtures/rss.json".to_string();
        let args = Args {
            private_key: None,
            relays: Some(relays_file_path),
            feeds: Some(feeds_file_path),
        };
        let config = NostrConfig::new(&args);
        assert_eq!(config.relays.len(), 4);
        assert_eq!(config.relays[0].name, String::from("noslol"));
        assert_eq!(config.relays[0].target, String::from("wss://nos.lol"));
        assert_eq!(config.relays[0].active, true);
        assert_eq!(config.relays[0].proxy, None);
    }

    #[test]
    fn test_nostrconfig_profile_setters() {
        use super::Relay;

        let relays_file_path = "./src/fixtures/relays.json".to_string();
        let feeds_file_path = "./src/fixtures/rss.json".to_string();
        let args = Args {
            private_key: None,
            relays: Some(relays_file_path),
            feeds: Some(feeds_file_path),
        };
        let mut config = NostrConfig::new(&args);

        config = config.set_banner(Some("https://domain.com/image.jpg".to_string()));
        assert_eq!(
            config.banner,
            Some("https://domain.com/image.jpg".to_string())
        );

        config = config.set_picture(Some("https://domain.com/image.jpg".to_string()));
        assert_eq!(
            config.picture,
            Some("https://domain.com/image.jpg".to_string())
        );

        config = config.set_name(Some("John doe".to_string()));
        assert_eq!(config.name, Some("John doe".to_string()));

        config = config.set_description(Some("Ad lorem ipsum".to_string()));
        assert_eq!(config.description, Some("Ad lorem ipsum".to_string()));

        let relays: Vec<Relay> = vec![Relay {
            name: "test".to_string(),
            target: "wss://localhost".to_string(),
            active: true,
            proxy: None,
        }];
        config = config.set_relays(relays);
        assert_eq!(config.description, Some("Ad lorem ipsum".to_string()));
    }
}
