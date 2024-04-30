#![allow(dead_code)]

use core::panic;
use log::warn;
use nostr_sdk::prelude::*;
use serde::{Deserialize, Serialize};
use std::{env, path::Path};

use crate::nostr::{relay::Relay, NostrProfile};

#[derive(Debug)]
pub enum ConfigErrors {
    FileLocationError,
    FileFormatError,
    FileParsingError,
    KeyParsingError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Profile {
    pub id: String,
    pub private_key: String,
    #[serde(default)]
    pub relays: Vec<Relay>,
    pub about: Option<String>,
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub picture: Option<String>,
    pub banner: Option<String>,
    pub nip05: Option<String>,
    pub lud16: Option<String>,
    #[serde(default = "Profile::default_pow_level")]
    pub pow_level: u8,
    #[serde(default)]
    pub recommended_relays: Option<Vec<String>>,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            private_key: Self::get_env_private_key(),
            id: "default".to_string(),
            relays: Vec::new(),
            about: Self::get_env_description(),
            name: Self::get_env_name(),
            display_name: Self::get_env_display_name(),
            description: Self::get_env_description(),
            picture: Self::get_env_picture(),
            banner: Self::get_env_banner(),
            nip05: Self::get_env_nip05(),
            lud16: Self::get_env_lud16(),
            pow_level: Self::default_pow_level(),
            recommended_relays: None,
        }
    }
}

impl NostrProfile for Profile {
    fn get_display_name(self) -> Option<String> {
        if self.display_name.is_some() {
            return self.display_name;
        }

        Self::get_env_display_name()
    }

    fn get_description(self) -> Option<String> {
        if self.description.is_some() {
            return self.description;
        }

        Self::get_env_description()
    }

    fn get_picture(self) -> Option<String> {
        if self.picture.is_some() {
            return self.picture;
        }

        Self::get_env_picture()
    }

    fn get_banner(self) -> Option<String> {
        if self.banner.is_some() {
            return self.banner;
        }

        Self::get_env_banner()
    }

    fn get_nip05(self) -> Option<String> {
        if self.nip05.is_some() {
            return self.nip05;
        }

        Self::get_env_nip05()
    }

    fn get_lud16(self) -> Option<String> {
        if self.lud16.is_some() {
            return self.lud16;
        }

        Self::get_env_lud16()
    }

    fn get_relays(&self) -> Vec<Relay> {
        self.relays.clone()
    }

    fn get_keys(&self) -> Keys {
        match Keys::parse(&self.private_key) {
            Ok(val) => val,
            Err(_) => {
                // warn!("Invalid private key found for Nostr. Generating random keys...");
                panic!("Invalid private key found. This should not happen.");
            }
        }
    }
}

impl Profile {
    pub fn new(private_key: String, relays: Option<String>) -> Self {
        let mut profile = Profile {
            private_key,
            ..Default::default()
        };

        if let Some(relays) = relays {
            profile = profile.load_relays(&relays);
        }

        profile
    }

    pub fn set_relays(mut self, relays: Vec<Relay>) -> Self {
        self.relays = relays;
        self
    }

    pub fn set_display_name(mut self, display_name: Option<String>) -> Self {
        self.display_name = display_name;
        self
    }

    fn get_env_display_name() -> Option<String> {
        match env::var("NOSTR_DISPLAY_NAME")
            .unwrap_or("".to_string())
            .parse::<String>()
        {
            Ok(result) => match !result.is_empty() {
                true => Some(result),
                false => None,
            },
            Err(_) => None,
        }
    }

    fn default_pow_level() -> u8 {
        env::var("DEFAULT_POW_LEVEL")
            .unwrap_or("0".to_string())
            .parse::<u8>()
            .unwrap_or(0)
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
            Ok(result) => match !result.is_empty() {
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

    fn get_env_description() -> Option<String> {
        match env::var("NOSTR_DESCRIPTION")
            .unwrap_or("".to_string())
            .parse::<String>()
        {
            Ok(result) => match !result.is_empty() {
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

    fn get_env_picture() -> Option<String> {
        match env::var("NOSTR_PICTURE")
            .unwrap_or("".to_string())
            .parse::<String>()
        {
            Ok(result) => match !result.is_empty() {
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

    fn get_env_banner() -> Option<String> {
        match env::var("NOSTR_BANNER")
            .unwrap_or("".to_string())
            .parse::<String>()
        {
            Ok(result) => match !result.is_empty() {
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

    fn get_env_nip05() -> Option<String> {
        match env::var("NOSTR_NIP05")
            .unwrap_or("".to_string())
            .parse::<String>()
        {
            Ok(result) => match !result.is_empty() {
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

    fn get_env_lud16() -> Option<String> {
        match env::var("NOSTR_LUD16")
            .unwrap_or("".to_string())
            .parse::<String>()
        {
            Ok(result) => match !result.is_empty() {
                true => Some(result),
                false => None,
            },
            Err(_) => None,
        }
    }

    pub fn set_recommended_relays(mut self, relays: Vec<String>) -> Self {
        self.recommended_relays = Some(relays);
        self
    }

    fn get_env_var(var_name: &str) -> Option<String> {
        match env::var(format!("NOSTR_{}", var_name.to_uppercase())) {
            Ok(val) => Some(val),
            Err(_) => None,
        }
    }

    pub fn set_keys(secret_key: &str) -> Result<Keys, ConfigErrors> {
        match Keys::parse(secret_key) {
            Ok(keys) => Ok(keys),
            Err(_) => Err(ConfigErrors::KeyParsingError),
        }
    }

    fn get_env_private_key() -> String {
        match env::var("NOSTR_PK") {
            Ok(val) => val,
            Err(_) => {
                warn!("No private key found for Nostr. Generating random keys...");
                panic!("No default profile key defined. Declare using NOSTR_PK in env file.");
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

    pub fn set_relays_from_file(self, path: &str) -> Self {
        self.load_relays(path)
    }
}

#[cfg(test)]
mod tests {

    use dotenv::from_filename;

    #[tokio::test]
    async fn test_default_profile() {
        from_filename(".env.test").ok();

        let profile = super::Profile {
            ..Default::default()
        };

        assert_eq!(profile.id, "default");
        assert_eq!(profile.display_name, Some("satoshi-nakamoto".to_string()));
        assert_eq!(
            profile.description,
            Some("Craig Wright is not satoshi".to_string())
        )
    }

    #[tokio::test]
    async fn test_new_profile() {
        from_filename(".env.test").ok();

        let profile = super::Profile::new("abcdef".to_string(), None);

        assert_eq!(profile.id, "default");
    }

    #[test]
    fn test_nostrconfig_profile_setters() {
        use super::Relay;

        from_filename(".env.test").ok();

        let mut profile = super::Profile::new("abcde".to_string(), None);

        profile = profile.set_banner(Some("https://domain.com/image.jpg".to_string()));
        assert_eq!(
            profile.banner,
            Some("https://domain.com/image.jpg".to_string())
        );

        profile = profile.set_picture(Some("https://domain.com/image.jpg".to_string()));
        assert_eq!(
            profile.picture,
            Some("https://domain.com/image.jpg".to_string())
        );

        profile = profile.set_name(Some("John doe".to_string()));
        assert_eq!(profile.name, Some("John doe".to_string()));

        profile = profile.set_description(Some("Ad lorem ipsum".to_string()));
        assert_eq!(profile.description, Some("Ad lorem ipsum".to_string()));

        let relays: Vec<Relay> = vec![Relay {
            name: "test".to_string(),
            target: "wss://localhost".to_string(),
            active: true,
            proxy: None,
            pow_level: 0,
        }];
        profile = profile.set_relays(relays);
        assert_eq!(profile.description, Some("Ad lorem ipsum".to_string()));
    }
}
