use nostrss_grpc::grpc::{
    self, DeleteProfileRequest, DeleteProfileResponse, ProfileInfoRequest, ProfileInfoResponse,
    ProfileItem, ProfilesListRequest, ProfilesListResponse,
};
use tokio::sync::MutexGuard;
use tonic::{Code, Request, Response, Status};

use crate::app::app::App;

pub struct ProfileRequestHandler {}

impl ProfileRequestHandler {
    // Interface to retrieve the list of profiles on instance
    pub async fn profiles_list(
        app: MutexGuard<'_, App>,
        _: Request<ProfilesListRequest>,
    ) -> Result<Response<ProfilesListResponse>, Status> {
        let mut profiles = Vec::new();

        for profile in app.profiles.clone() {
            profiles.push(ProfileItem::from(profile.1));
        }

        Ok(Response::new(grpc::ProfilesListResponse { profiles }))
    }

    // Interface to delete a profile on instance
    pub async fn delete_profile(
        mut app: MutexGuard<'_, App>,
        request: Request<DeleteProfileRequest>,
    ) -> Result<Response<DeleteProfileResponse>, Status> {
        let profile_id = &request.into_inner().id;
        let client = app.clients.remove(profile_id.trim());

        if client.is_none() {
            return Err(Status::new(Code::NotFound, "No profile with that id found"));
        }

        Ok(Response::new(grpc::DeleteProfileResponse {}))
    }

    // Interface to retrieve the detailed configuration of a single profile on instance
    pub async fn profile_info(
        app: MutexGuard<'_, App>,
        request: Request<ProfileInfoRequest>,
    ) -> Result<Response<ProfileInfoResponse>, Status> {
        let id = &request.into_inner().id;
        match app.clients.get(id.trim()) {
            Some(client) => Ok(Response::new(ProfileInfoResponse {
                profile: ProfileItem::from(client.config.clone()),
            })),
            None => {
                return Err(Status::new(Code::NotFound, "Profile not found"));
            }
        }
    }
}
