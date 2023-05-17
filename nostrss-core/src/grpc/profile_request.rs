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

#[cfg(test)]
mod tests {

    use super::*;
    use std::sync::Arc;

    use crate::grpc::grpctest_utils::mock_app;
    use tokio::sync::Mutex;
    use tonic::Request;

    #[tokio::test]
    async fn add_profile_test() {}

    #[tokio::test]
    async fn list_profiles_test() {
        let app = Arc::new(Mutex::new(mock_app().await));

        let profiles_list_request = ProfilesListRequest {};
        let request = Request::new(profiles_list_request);

        let profiles_list_request_result =
            ProfileRequestHandler::profiles_list(app.lock().await, request).await;

        assert_eq!(profiles_list_request_result.is_ok(), true);

        let response = profiles_list_request_result.unwrap().into_inner();

        assert_eq!(response.profiles.len(), 2);
    }

    #[tokio::test]
    async fn delete_profile_test() {
        let app = Arc::new(Mutex::new(mock_app().await));

        let delete_profile_request = DeleteProfileRequest {
            id: "test".to_string(),
        };
        let request = Request::new(delete_profile_request);

        let delete_profile_request_result =
            ProfileRequestHandler::delete_profile(app.lock().await, request).await;

        assert_eq!(delete_profile_request_result.is_ok(), true);
    }

    #[tokio::test]
    async fn profile_info_test() {
        let app = Arc::new(Mutex::new(mock_app().await));

        let profiles_list_request = ProfilesListRequest {};
        let request = Request::new(profiles_list_request);

        let profiles_list_request_result =
            ProfileRequestHandler::profiles_list(app.lock().await, request).await;

        assert_eq!(profiles_list_request_result.is_ok(), true);
    }
}
