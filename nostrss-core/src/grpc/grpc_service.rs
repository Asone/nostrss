#[allow(implied_bounds_entailment)]
use std::sync::Arc;

use nostr_sdk::{
    prelude::{FromSkStr, ToBech32},
    Keys,
};
use nostrss_grpc::grpc::{
    self, nostrss_grpc_server::NostrssGrpc, DeleteFeedRequest, DeleteFeedResponse,
    DeleteProfileRequest, DeleteProfileResponse, FeedInfoRequest, FeedInfoResponse, FeedItem,
    FeedsListRequest, FeedsListResponse, ProfileInfoRequest, ProfileInfoResponse, ProfileItem,
    ProfilesListRequest, ProfilesListResponse, StartJobRequest, StartJobResponse, StateRequest,
    StateResponse, StopJobRequest, StopJobResponse,
};
use tokio::sync::Mutex;
use tonic::{Code, Request, Response, Status};

use crate::{app::app::App, profiles::config::Profile, rss::config::Feed};

/// Provides the gRPC service handling that allows
/// remote operations.
pub struct NostrssServerService {
    pub app: Arc<Mutex<App>>,
}

impl From<Profile> for ProfileItem {
    fn from(value: Profile) -> Self {
        let public_key = match Keys::from_sk_str(&value.private_key) {
            Ok(keys) => keys.public_key().to_bech32().unwrap(),
            Err(_) => "".to_string(),
        };

        Self {
            id: value.id,
            public_key: public_key,
            name: value.name,
            relays: Vec::new(),
            display_name: value.display_name,
            description: value.description,
            picture: value.picture,
            banner: value.banner,
            nip05: value.nip05,
            lud16: value.lud16,
            pow_level: Some(0),
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

        let cache_size = value.cache_size as u64;
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

#[tonic::async_trait]
impl NostrssGrpc for NostrssServerService {
    // Retrieves state of the core nostrss application
    async fn state(
        &self,
        request: Request<StateRequest>,
    ) -> Result<Response<StateResponse>, Status> {
        let app_lock = self.app.lock().await;
        let n = app_lock.profiles.keys().len();
        let _ = request.into_inner();
        Ok(Response::new(grpc::StateResponse {
            state: format!("App is alive. Number of profiles : {}", n),
        }))
    }

    // Interface to retrieve the list of profiles on instance
    async fn profiles_list(
        &self,
        _: Request<ProfilesListRequest>,
    ) -> Result<Response<ProfilesListResponse>, Status> {
        let app_lock = self.app.lock().await;
        let mut profiles = Vec::new();

        for profile in app_lock.profiles.clone() {
            profiles.push(ProfileItem::from(profile.1));
        }

        Ok(Response::new(grpc::ProfilesListResponse { profiles }))
    }

    // Interface to retrieve the list of feed on instance
    async fn feeds_list(
        &self,
        _: Request<FeedsListRequest>,
    ) -> Result<Response<FeedsListResponse>, Status> {
        let app_lock = self.app.lock().await;
        let mut feeds = Vec::new();

        for feed in app_lock.rss.feeds.clone() {
            let f = FeedItem::from(feed);
            feeds.push(f);
        }

        Ok(Response::new(grpc::FeedsListResponse { feeds }))
    }

    // Interface to delete a feed on instance
    async fn delete_feed(
        &self,
        request: Request<DeleteFeedRequest>,
    ) -> Result<Response<DeleteFeedResponse>, Status> {
        let app_lock = self.app.lock().await;
        let feed_id = &request.into_inner().id;
        let job_uuid = app_lock.feeds_jobs.get(feed_id.trim());

        if job_uuid.is_none() {
            return Err(Status::new(
                Code::NotFound,
                "Job associated to feed not found",
            ));
        }

        _ = app_lock.scheduler.remove(job_uuid.unwrap());
        Ok(Response::new(grpc::DeleteFeedResponse {}))
    }

    // Interface to delete a profile on instance
    async fn delete_profile(
        &self,
        request: Request<DeleteProfileRequest>,
    ) -> Result<Response<DeleteProfileResponse>, Status> {
        let mut app_lock = self.app.lock().await;
        let profile_id = &request.into_inner().id;
        let client = app_lock.clients.remove(profile_id.trim());

        if client.is_none() {
            return Err(Status::new(Code::NotFound, "No profile with that id found"));
        }

        Ok(Response::new(grpc::DeleteProfileResponse {}))
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

    async fn feed_info(
        &self,
        request: Request<FeedInfoRequest>,
    ) -> Result<Response<FeedInfoResponse>, Status> {
        let app_lock = self.app.lock().await;
        let id = &request.into_inner().id;
        match app_lock.rss.feeds.clone().into_iter().find(|f| &f.id == id) {
            Some(feed) => Ok(Response::new(FeedInfoResponse {
                feed: Some(FeedItem::from(feed)),
            })),
            None => {
                return Err(Status::new(Code::NotFound, "Feed not found"));
            }
        }
    }

    // Interface to retrieve the detailed configuration of a single profile on instance
    async fn profile_info(
        &self,
        request: Request<ProfileInfoRequest>,
    ) -> Result<Response<ProfileInfoResponse>, Status> {
        let app_lock = self.app.lock().await;
        let id = &request.into_inner().id;
        match app_lock.clients.get(id.trim()) {
            Some(client) => Ok(Response::new(ProfileInfoResponse {
                profile: Some(ProfileItem::from(client.config.clone())),
            })),
            None => {
                return Err(Status::new(Code::NotFound, "Profile not found"));
            }
        }
    }
}
