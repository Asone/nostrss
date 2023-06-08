#![allow(dead_code)]

use std::{collections::HashMap, fs::File, path::Path, str::FromStr};

use log::error;
use reqwest::Url;

use crate::nostr::relay::Relay;

use super::config::Profile;

#[derive(Debug, PartialEq, Clone)]
pub struct ProfileHandler(pub HashMap<String, Profile>);

impl ProfileHandler {
    pub fn new(path: &Option<String>, default_relays: &str) -> Self {
        // Init profile instances index
        let mut profiles = Self(HashMap::new());

        // Register default profile
        let mut default_profile = Profile::default();
        default_profile = default_profile.set_relays_from_file(default_relays);

        profiles
            .0
            .insert(default_profile.clone().id, default_profile);

        if let Some(path) = path {
            profiles = profiles.load_profiles(path);
        };

        profiles
    }

    pub fn save_profiles(self, path: &str, profiles: Vec<&Profile>) -> bool {
        let path = Path::new(path);

        if path.is_file() {
            match path.extension() {
                Some(ext) => match ext.to_str() {
                    Some("yml") => {
                        return self.save_yaml_profiles(path, profiles);
                    }
                    Some("yaml") => {
                        return self.save_yaml_profiles(path, profiles);
                    }
                    Some("json") => {
                        return self.save_json_profiles(path, profiles);
                    }
                    _ => {
                        return false;
                    }
                },
                None => {
                    return false;
                }
            }
        }

        false
    }

    pub fn save_json_profiles(self, path: &Path, profiles: Vec<&Profile>) -> bool {
        // let serialized = serde_json::to_string(&profiles).unwrap();

        // let result = serde_json::to_writer_pretty(path, &profiles);

        let file = File::create(path).unwrap();
        let writer = std::io::BufWriter::new(file);
        let result = serde_json::to_writer_pretty(writer, &profiles);

        match result {
            Ok(_) => true,
            Err(e) => {
                error!("{}", e);
                false
            }
        }
    }

    pub fn save_yaml_profiles(self, path: &Path, profiles: Vec<&Profile>) -> bool {
        // let serialized = serde_json::to_string(&profiles).unwrap();

        // let result = serde_json::to_writer_pretty(path, &profiles);

        let file = File::create(path).unwrap();
        let writer = std::io::BufWriter::new(file);
        let result = serde_yaml::to_writer(writer, &profiles);

        match result {
            Ok(_) => true,
            Err(e) => {
                error!("{}", e);
                false
            }
        }
    }

    pub fn load_profiles(self, path: &str) -> Self {
        let path = Path::new(path);

        if path.is_file() {
            match path.extension() {
                Some(ext) => match ext.to_str() {
                    Some("yml") => {
                        return self.load_yaml_profiles(path);
                    }
                    Some("yaml") => {
                        return self.load_yaml_profiles(path);
                    }
                    Some("json") => {
                        return self.load_json_profiles(path);
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

    pub fn load_json_profiles(mut self, path: &Path) -> Self {
        let file = match std::fs::File::open(path) {
            Ok(file) => file,
            Err(_) => {
                error!("Profiles file not found");
                return self;
            }
        };

        let profiles: Vec<Profile> = match serde_json::from_reader(file) {
            Ok(profiles) => profiles,
            Err(e) => {
                error!("Invalid Profiles file");
                error!("{}", e);
                return self;
            }
        };

        self.0.extend(Self::profiles_vec_to_hashmap(profiles));
        self
    }

    fn load_yaml_profiles(mut self, path: &Path) -> Self {
        let file = match std::fs::File::open(path) {
            Ok(file) => file,
            Err(_) => {
                error!("Profiles file not found");
                return self;
            }
        };
        let profiles: Vec<Profile> = match serde_yaml::from_reader(file) {
            Ok(profiles) => profiles,
            Err(_) => {
                error!("Invalid Profiles file");
                return self;
            }
        };

        self.0.extend(Self::profiles_vec_to_hashmap(profiles));
        self
    }

    fn profiles_vec_to_hashmap(profiles: Vec<Profile>) -> HashMap<String, Profile> {
        let mut profiles_hashmap = HashMap::new();

        for profile in profiles {
            profiles_hashmap.insert(profile.id.clone(), profile);
        }

        profiles_hashmap
    }

    pub fn get_profiles(&self) -> HashMap<String, Profile> {
        self.0.clone()
    }

    pub fn get_default(self) -> Profile {
        self.0["default"].clone()
    }

    pub fn get(&self, id: &String) -> Option<Profile> {
        let result = self.0[id].clone();
        if &result.id != id {
            return None;
        }
        Some(result)
    }

    pub fn get_default_relays(self) -> Vec<Relay> {
        let default_profile = self.0["default"].clone();
        default_profile.relays
    }

    pub fn new_get_default_relays(&self) -> HashMap<Url, Relay> {
        self.0["default"]
            .clone()
            .relays
            .into_iter()
            .map(|r| (Url::from_str(r.target.as_str()).unwrap(), r))
            .collect()
    }
}

#[cfg(test)]
mod tests {

    use dotenv::from_filename;

    use super::*;

    #[tokio::test]
    async fn test_default_profile_handler() {
        from_filename(".env.test").ok();

        let relays_path = "src/fixtures/relays.json".to_string();

        let profile_handler = ProfileHandler::new(&None, &relays_path);

        assert_eq!(profile_handler.0.keys().len(), 1);
    }
    #[tokio::test]
    async fn test_profile_handler_with_yaml_file() {
        from_filename(".env.test").ok();

        let relays_path = "src/fixtures/relays.json".to_string();
        let profiles_path = "src/fixtures/profiles.yaml".to_string();

        let profile_handler = ProfileHandler::new(&Some(profiles_path), &relays_path);

        let profiles_size = profile_handler.0.keys().len();
        assert_eq!(profiles_size, 3);
    }

    #[tokio::test]
    async fn test_profile_handler_with_json_file() {
        from_filename(".env.test").ok();

        let relays_path = "src/fixtures/relays.json".to_string();
        let profiles_path = "src/fixtures/profiles.json".to_string();

        let profile_handler = ProfileHandler::new(&Some(profiles_path), &relays_path);

        let profiles_size = profile_handler.0.keys().len();
        assert_eq!(profiles_size, 3);
    }

    #[tokio::test]
    async fn test_get() {
        from_filename(".env.test").ok();

        let relays_path = "src/fixtures/relays.json".to_string();

        let profile_handler = ProfileHandler::new(&None, &relays_path);
        let profile = profile_handler.get(&"default".to_string());

        assert_eq!(&profile.is_some(), &true);
    }

    #[tokio::test]
    async fn test_profiles_vec_to_hashmap() {
        from_filename(".env.test").ok();

        let mut profiles = Vec::new();
        let profile = Profile {
            ..Default::default()
        };
        let _ = &profiles.push(profile.clone());

        let hashmap = ProfileHandler::profiles_vec_to_hashmap(profiles);

        assert_eq!(&hashmap.keys().len(), &1);
        assert_eq!(hashmap["default"], profile)
    }
}
