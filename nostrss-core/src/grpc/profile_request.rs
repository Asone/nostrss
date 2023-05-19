use nostrss_grpc::grpc::{
    self, AddProfileRequest, AddProfileResponse, DeleteProfileRequest, DeleteProfileResponse,
    NewProfileItem, ProfileInfoRequest, ProfileInfoResponse, ProfileItem, ProfilesListRequest,
    ProfilesListResponse,
};
use tokio::sync::MutexGuard;
use tonic::{Code, Request, Response, Status};

use crate::{app::app::App, nostr::nostr::NostrInstance, profiles::config::Profile};

impl From<NewProfileItem> for Profile {
    fn from(value: NewProfileItem) -> Self {
        let pow_level = match value.pow_level {
            Some(value) => value as u8,
            None => 0,
        };

        Self {
            id: value.id,
            private_key: value.private_key,
            relays: Vec::new(),
            about: value.description.clone(),
            name: value.name,
            display_name: value.display_name,
            description: value.description,
            picture: value.picture,
            banner: value.banner,
            nip05: value.nip05,
            lud16: value.lud16,
            pow_level: pow_level,
            recommended_relays: Some(value.recommended_relays),
        }
    }
}

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

    pub async fn add_profile(
        mut app: MutexGuard<'_, App>,
        request: Request<AddProfileRequest>,
    ) -> Result<Response<AddProfileResponse>, Status> {
        let new_profile_item = request.into_inner().profile;

        let profile = Profile::from(new_profile_item);

        let client = NostrInstance::new(profile.clone()).await;
        app.profiles.insert(profile.id.clone(), profile.clone());
        app.clients.insert(profile.id.clone(), client);

        Ok(Response::new(grpc::AddProfileResponse {}))
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
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::sync::Arc;

    use crate::grpc::grpctest_utils::mock_app;
    use dotenv::from_filename;
    use nostrss_grpc::grpc::{AddProfileRequest, NewProfileItem};
    use tokio::sync::Mutex;
    use tonic::Request;

    #[tokio::test]
    async fn add_profile_test() {
        let app = Arc::new(Mutex::new(mock_app().await));

        let add_profile_request = AddProfileRequest {
            profile: NewProfileItem {
                id: "added".to_string(),
                private_key: "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
                    .to_string(),
                ..Default::default()
            },
        };

        let request = Request::new(add_profile_request);

        let profile_add_request_result = {
            let app_lock = app.lock().await;
            ProfileRequestHandler::add_profile(app_lock, request).await
        };

        assert_eq!(profile_add_request_result.is_ok(), true);

        let profiles_list_request = ProfilesListRequest {};
        let request = Request::new(profiles_list_request);

        let profiles_list_request_result = {
            let app_lock = app.lock().await;
            ProfileRequestHandler::profiles_list(app_lock, request).await
        };

        let response = profiles_list_request_result.unwrap().into_inner();

        assert_eq!(response.profiles.len(), 3);
    }

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

        let response = profiles_list_request_result.unwrap().into_inner();

        assert_eq!(response.profiles.len(), 2);
    }
}
