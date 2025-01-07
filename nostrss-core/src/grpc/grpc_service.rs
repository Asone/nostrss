use std::str::FromStr;
use std::sync::Arc;

use nostr_sdk::{prelude::ToBech32, Keys};

use crate::rss::config::Feed;
use nostrss_grpc::grpc::{
    self, nostrss_grpc_server::NostrssGrpc, AddFeedRequest, AddFeedResponse, AddProfileRequest,
    AddProfileResponse, DeleteFeedRequest, DeleteFeedResponse, DeleteProfileRequest,
    DeleteProfileResponse, FeedInfoRequest, FeedInfoResponse, FeedItem, FeedsListRequest,
    FeedsListResponse, ProfileInfoRequest, ProfileInfoResponse, ProfileItem, ProfilesListRequest,
    ProfilesListResponse, StartJobRequest, StartJobResponse, StateRequest, StateResponse,
    StopJobRequest, StopJobResponse,
};
use tokio::sync::{Mutex, MutexGuard};
use tonic::{Request, Response, Status};

use crate::{app::app::App, profiles::config::Profile};

use super::{feed_request::FeedRequestHandler, profile_request::ProfileRequestHandler};

/// Provides the gRPC service handling that allows
/// remote operations.
pub struct NostrssServerService {
    pub app: Arc<Mutex<App>>,
}

impl From<FeedItem> for Feed {
    fn from(value: FeedItem) -> Self {
        let url = value.url.as_str();

        let cache_size = match value.cache_size {
            Some(r) => match usize::try_from(r) {
                Ok(result) => Some(result),
                Err(_) => Self::default_cache_size(),
            },
            None => None,
        };
        // let cache_size = match usize::try_from(value.cache_size) {
        //     Ok(result) => Some(result),
        //     Err(_) => Self::default_cache_size(),
        // };

        let pow_level = match u8::try_from(value.pow_level) {
            Ok(result) => result,
            Err(_) => Self::default_pow_level(),
        };

        Self {
            id: value.id,
            name: value.name,
            url: nostr_sdk::Url::from_str(url).unwrap(),
            schedule: value.schedule,
            profiles: Some(value.profiles),
            tags: Some(value.tags),
            template: value.template,
            cache_size,
            pow_level,
        }
    }
}

impl From<Profile> for ProfileItem {
    fn from(value: Profile) -> Self {
        let public_key = match Keys::parse(&value.private_key) {
            Ok(keys) => keys.public_key().to_bech32().unwrap(),
            Err(_) => "".to_string(),
        };

        Self {
            id: value.id,
            public_key,
            name: value.name,
            relays: Vec::new(),
            display_name: value.display_name,
            description: value.description,
            picture: value.picture,
            banner: value.banner,
            nip05: value.nip05,
            lud16: value.lud16,
            pow_level: Some(value.pow_level.into()),
            recommended_relays: Vec::new(),
        }
    }
}

impl From<Feed> for FeedItem {
    fn from(value: Feed) -> FeedItem {
        let profiles = match value.profiles {
            Some(profiles) => profiles,
            None => Vec::new(),
        };

        let tags = match value.tags {
            Some(t) => t,
            None => Vec::new(),
        };

        let cache_size = match value.cache_size {
            Some(r) => Some(r as u64),
            None => None,
        };
        let pow_level = value.pow_level as u64;

        FeedItem {
            id: value.id,
            name: value.name,
            url: value.url.to_string(),
            schedule: value.schedule,
            profiles,
            tags,
            template: value.template,
            cache_size,
            pow_level,
        }
    }
}

impl NostrssServerService {
    async fn get_app_lock(&self) -> MutexGuard<App> {
        self.app.lock().await
    }
}

#[tonic::async_trait]
impl NostrssGrpc for NostrssServerService {
    // Retrieves state of the core nostrss application
    async fn state(
        &self,
        request: Request<StateRequest>,
    ) -> Result<Response<StateResponse>, Status> {
        let app_lock = self.app.lock().await;
        let n = app_lock.nostr_service.profiles.keys().len();
        let _ = request.into_inner();
        Ok(Response::new(grpc::StateResponse {
            state: format!("App is alive. Number of profiles : {}", n),
        }))
    }

    // Interface to retrieve the list of feed on instance
    async fn feeds_list(
        &self,
        request: Request<FeedsListRequest>,
    ) -> Result<Response<FeedsListResponse>, Status> {
        FeedRequestHandler::feeds_list(self.get_app_lock().await, request).await
    }

    async fn feed_info(
        &self,
        request: Request<FeedInfoRequest>,
    ) -> Result<Response<FeedInfoResponse>, Status> {
        FeedRequestHandler::feed_info(self.get_app_lock().await, request).await
    }
    async fn add_feed(
        &self,
        request: Request<AddFeedRequest>,
    ) -> Result<Response<AddFeedResponse>, Status> {
        FeedRequestHandler::add_feed(self.get_app_lock().await, request).await
    }

    // Interface to delete a feed on instance
    async fn delete_feed(
        &self,
        request: Request<DeleteFeedRequest>,
    ) -> Result<Response<DeleteFeedResponse>, Status> {
        FeedRequestHandler::delete_feed(self.get_app_lock().await, request).await
    }

    // Interface to retrieve the list of profiles on instance
    async fn profiles_list(
        &self,
        request: Request<ProfilesListRequest>,
    ) -> Result<Response<ProfilesListResponse>, Status> {
        ProfileRequestHandler::profiles_list(self.get_app_lock().await, request).await
    }

    // Interface to retrieve the detailed configuration of a single profile on instance
    async fn profile_info(
        &self,
        request: Request<ProfileInfoRequest>,
    ) -> Result<Response<ProfileInfoResponse>, Status> {
        ProfileRequestHandler::profile_info(self.get_app_lock().await, request).await
    }

    // Interface to delete a profile on instance
    async fn add_profile(
        &self,
        request: Request<AddProfileRequest>,
    ) -> Result<Response<AddProfileResponse>, Status> {
        ProfileRequestHandler::add_profile(self.get_app_lock().await, request).await
    }

    // Interface to delete a profile on instance
    async fn delete_profile(
        &self,
        request: Request<DeleteProfileRequest>,
    ) -> Result<Response<DeleteProfileResponse>, Status> {
        ProfileRequestHandler::delete_profile(self.get_app_lock().await, request).await
    }

    // Interface to start a job on instance
    async fn start_job(
        &self,
        request: Request<StartJobRequest>,
    ) -> Result<Response<StartJobResponse>, Status> {
        let _app_lock = self.app.lock().await;
        let _feed_id = &request.into_inner().feed_id;

        Ok(Response::new(grpc::StartJobResponse {}))
    }

    // Interface to retrieve a job instance
    // This should be renamed `stop_jobs` and
    // should only shutdown the scheduler
    async fn stop_job(
        &self,
        request: Request<StopJobRequest>,
    ) -> Result<Response<StopJobResponse>, Status> {
        let _app_lock = self.app.lock().await;
        let _feed_id = &request.into_inner().feed_id;

        Ok(Response::new(grpc::StopJobResponse {}))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::rss::config::Feed;
    use nostrss_grpc::grpc::AddFeedRequest;

    #[test]
    fn feed_from_add_feed_request_test() {
        let request = AddFeedRequest {
            feed: FeedItem {
                id: "test".to_string(),
                name: "test".to_string(),
                url: "https://myrss.rs".to_string(),
                schedule: "1/10 * * * * *".to_string(),
                profiles: Vec::new(),
                tags: Vec::new(),
                template: None,
                cache_size: Some(10),
                pow_level: 20,
            },
            save: Some(false),
        };

        let feed = Feed::from(request.feed);

        let expected = "test";
        assert_eq!(feed.id.as_str(), expected);

        let expected = "test";
        assert_eq!(feed.name.as_str(), expected);

        let expected = "https://myrss.rs/";
        let url = feed.url.as_str();
        assert_eq!(url, expected);
    }

    #[test]
    fn profile_item_from_profile_test() {
        let profile = Profile {
            id: "test".to_string(),
            private_key: "6789abcdef0123456789abcdef0123456789abcdef0123456789abcdef012345"
                .to_string(),
            relays: Vec::new(),
            about: Some("Ad lorem ipsum".to_string()),
            name: Some("Some test account".to_string()),
            display_name: Some("Some test account display name".to_string()),
            description: Some("Ad lorem ipsum description".to_string()),
            picture: Some("http://myimage.jpg".to_string()),
            banner: None,
            nip05: None,
            lud16: None,
            pow_level: 23,
            recommended_relays: Some(Vec::new()),
        };

        let profile_item = ProfileItem::from(profile.clone());

        assert_eq!(profile_item.id, profile.id);

        let keys = Keys::parse(profile.private_key.as_str()).unwrap();

        assert_eq!(
            profile_item.public_key,
            keys.public_key().to_bech32().unwrap()
        );

        assert_eq!(profile_item.banner, None);
        assert_eq!(profile_item.pow_level, Some(23));
    }

    #[test]
    fn feed_item_from_feed_test() {
        let feed = Feed {
            id: "test".to_string(),
            name: "My test".to_string(),
            url: nostr_sdk::Url::from_str("https://myrss.rss").unwrap(),
            schedule: "1/10 * * * * *".to_string(),
            ..Default::default()
        };

        let feed_item = FeedItem::from(feed.clone());

        assert_eq!(feed_item.id, feed.id);
        assert_eq!(feed_item.url.as_str(), feed.url.as_str());
    }
}
