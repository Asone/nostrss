use std::{path::Path, collections::HashMap};

use log::{ info, error};
use serde::{ Serialize, Deserialize };

use crate::config::Args;

#[derive(Debug,PartialEq,Serialize,Deserialize)]
struct Profile {
    id: String,
    private_key: String,
    relays: Option<String>,
    about: Option<String>,
    name: Option<String>,
    display_name: Option<String>,
    description: Option<String>,
    picture: Option<String>,
    banner: Option<String>,
    nip05: Option<String>,
    lud16: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct ProfileHandler(HashMap<String,Profile>);

impl ProfileHandler {
    pub fn new(args: &Args) -> Self {

        let mut profiles = Self(HashMap::new());

        if args.feeds.is_some() {
            info!("Found Rss file path argument. Parsing file...");
            profiles = profiles.load_profiles(&args.feeds.as_ref().unwrap());
        }

        profiles
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
            Err(_) => {
                error!("Invalid Profiles file");
                return self;
            }
        };

        self.0 = Self::profiles_vec_to_hashmap(profiles);
        self
    }

    pub fn load_yaml_profiles(mut self, path: &Path) -> Self {
        let file = match std::fs::File::open(path) {
            Ok(file) => file,
            Err(_) => {
                error!("Feeds file not found");
                return self;
            }
        };
        let profiles: Vec<Profile> = match serde_yaml::from_reader(file) {
            Ok(profiles) => profiles,
            Err(_) => {
                error!("Invalid Feed file");
                return self;
            }
        };


        self.0 = Self::profiles_vec_to_hashmap(profiles);
        self
    }

    fn profiles_vec_to_hashmap(profiles: Vec<Profile>) -> HashMap<String, Profile> {
        let mut profiles_hashmap = HashMap::new();

        for profile in profiles {
            profiles_hashmap.insert(profile.id.clone(),profile);
        
        };

        profiles_hashmap
    }


}

mod tests {

    use super::*;

    // #[test]
    // fn test_get_profile() -> Option<Profile> {
    //     None
    // }

} 
