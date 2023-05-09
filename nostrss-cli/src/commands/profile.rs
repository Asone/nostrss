#![allow(dead_code)]

use clap::{Parser, ValueEnum};
use nostrss_grpc::grpc::{
    nostrss_grpc_client::NostrssGrpcClient, DeleteProfileRequest, ProfileInfoRequest, ProfileItem,
    ProfilesListRequest,
};
use tabled::{Table, Tabled};
use tonic::{async_trait, transport::Channel};

use super::CommandsHandler;

#[derive(Clone, PartialEq, Parser, Debug, ValueEnum)]
pub enum ProfileActions {
    Add,
    Delete,
    List,
    Info,
}

pub struct ProfileCommandsHandler {
    pub client: NostrssGrpcClient<Channel>,
}

#[derive(Tabled)]
pub struct ProfileDetailsTemplate {
    pub key: String,
    pub value: String,
}

#[derive(Tabled)]
pub struct ListProfileTemplate {
    pub id: String,
    pub name: String,
    pub public_key: String,
}

impl From<ProfileItem> for ListProfileTemplate {
    fn from(data: ProfileItem) -> Self {
        Self {
            id: data.id,
            public_key: data.public_key,
            name: data.name.unwrap_or_else(|| "".to_string()),
        }
    }
}

#[derive(Tabled)]
pub struct FullProfileTemplate {
    pub id: String,
    pub public_key: String,
    pub name: String,
    pub relays: String,
    pub display_name: String,
    pub description: String,
    pub picture: String,
    pub banner: String,
    pub nip05: String,
    pub lud16: String,
    pub pow_level: i32,
    pub recommended_relays: String,
}

impl FullProfileTemplate {
    // Builds a table row from each property of profile struct
    pub fn properties_to_vec(&self) -> Vec<ProfileDetailsTemplate> {
        let pow_level = &self.pow_level.to_string();

        let properties: Vec<(String, &String)> = [
            ("id".to_string(), &self.id),
            ("public_key".to_string(), &self.public_key),
            ("name".to_string(), &self.name),
            ("relays".to_string(), &self.relays),
            ("display_name".to_string(), &self.display_name),
            ("description".to_string(), &self.description),
            ("picture".to_string(), &self.picture),
            ("banner".to_string(), &self.banner),
            ("nip05".to_string(), &self.nip05),
            ("lud16".to_string(), &self.lud16),
            ("pow_level".to_string(), pow_level),
            ("recommended_relays".to_string(), &self.recommended_relays),
        ]
        .to_vec();

        properties
            .into_iter()
            .map(|p| ProfileDetailsTemplate {
                key: p.0,
                value: p.1.to_string(),
            })
            .collect()
    }
}

impl From<ProfileItem> for FullProfileTemplate {
    fn from(data: ProfileItem) -> Self {
        Self {
            id: data.id,
            public_key: data.public_key,
            name: data.name.unwrap_or_else(|| "".to_string()),
            relays: data.relays.join(","),
            display_name: data.display_name.unwrap_or_else(|| "".to_string()),
            description: data.description.unwrap_or_else(|| "".to_string()),
            picture: data.picture.unwrap_or_else(|| "".to_string()),
            banner: data.banner.unwrap_or_else(|| "".to_string()),
            nip05: data.nip05.unwrap_or_else(|| "".to_string()),
            lud16: data.lud16.unwrap_or_else(|| "".to_string()),
            pow_level: data.pow_level.unwrap_or_else(|| 0),
            recommended_relays: data.recommended_relays.join(","),
        }
    }
}

impl FullProfileTemplate {
    fn new(
        id: String,
        public_key: String,
        name: String,
        relays: Vec<String>,
        display_name: Option<String>,
        description: Option<String>,
        picture: Option<String>,
        banner: Option<String>,
        nip05: Option<String>,
        lud16: Option<String>,
        pow_level: Option<i32>,
        recommended_relays: Vec<String>,
    ) -> Self {
        Self {
            id,
            public_key,
            name,
            relays: relays.join(","),
            display_name: display_name.unwrap_or_else(|| "".to_string()),
            description: description.unwrap_or_else(|| "".to_string()),
            picture: picture.unwrap_or_else(|| "".to_string()),
            banner: banner.unwrap_or_else(|| "".to_string()),
            nip05: nip05.unwrap_or_else(|| "".to_string()),
            lud16: lud16.unwrap_or_else(|| "".to_string()),
            pow_level: pow_level.unwrap_or_else(|| 0),
            recommended_relays: recommended_relays.join(","),
        }
    }
}
#[async_trait]
impl CommandsHandler for ProfileCommandsHandler {}

impl ProfileCommandsHandler {
    pub async fn handle(&mut self, action: ProfileActions) {
        match action {
            ProfileActions::Add => self.add(),
            ProfileActions::Delete => self.delete().await,
            ProfileActions::List => self.list().await,
            ProfileActions::Info => self.info().await,
        }
    }

    async fn list(&mut self) {
        // Case logic should come here
        let request = tonic::Request::new(ProfilesListRequest {});
        let response = self.client.profiles_list(request).await;
        match response {
            Ok(response) => {
                let raws: Vec<ListProfileTemplate> = response
                    .into_inner()
                    .profiles
                    .into_iter()
                    .map(|p| ListProfileTemplate::from(p))
                    .collect();

                let table = Table::new(raws);
                println!("=== Profiles list ===");
                println!("{}", table.to_string());
            }
            Err(e) => {
                println!("Error {}: {}", e.code(), e.message());
            }
        }
    }

    fn add(&self) {}

    async fn delete(&mut self) {
        let id = self.get_input("Id: ");
        let request = tonic::Request::new(DeleteProfileRequest { id });
        let response = self.client.delete_profile(request).await;

        match response {
            Ok(_) => {
                println!("Profile successfully deleted");
            }
            Err(e) => {
                println!(
                    "An error happened with code {} : {} ",
                    e.code(),
                    e.message()
                );
            }
        }
    }

    async fn info(&mut self) {
        let id = self.get_input("Id: ");

        let request = tonic::Request::new(ProfileInfoRequest { id });
        let response = self.client.profile_info(request).await;

        match response {
            Ok(response) => {
                match response.into_inner().profile {
                    Some(profile) => {
                        let profile = FullProfileTemplate::from(profile);
                        // profile.fields()
                        let table = Table::new(profile.properties_to_vec());
                        println!("{}", table.to_string());
                    }
                    None => {
                        println!("No profile found for this id");
                    }
                }
            }
            Err(e) => {
                println!(
                    "An error happened with code {} : {} ",
                    e.code(),
                    e.message()
                );
            }
        }
    }
}
