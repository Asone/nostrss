use nostrss_grpc::grpc::{
    self, AddFeedRequest, AddFeedResponse, DeleteFeedRequest, DeleteFeedResponse, FeedInfoRequest,
    FeedInfoResponse, FeedItem, FeedsListRequest, FeedsListResponse,
};
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};
use tonic::{Code, Request, Response, Status};

use crate::{app::app::App, rss::config::Feed, scheduler::scheduler::schedule};

pub struct FeedRequestHandler {}

impl FeedRequestHandler {
    pub async fn feeds_list(
        app: MutexGuard<'_, App>,
        _: Request<FeedsListRequest>,
    ) -> Result<Response<FeedsListResponse>, Status> {
        let mut feeds = Vec::new();

        for feed in app.rss.feeds.clone() {
            let f = FeedItem::from(feed);
            feeds.push(f);
        }

        Ok(Response::new(grpc::FeedsListResponse { feeds }))
    }

    pub async fn feed_info(
        app: MutexGuard<'_, App>,
        request: Request<FeedInfoRequest>,
    ) -> Result<Response<FeedInfoResponse>, Status> {
        let id = &request.into_inner().id;
        match app.rss.feeds.clone().into_iter().find(|f| &f.id == id) {
            Some(feed) => Ok(Response::new(FeedInfoResponse {
                feed: FeedItem::from(feed),
            })),
            None => Err(Status::new(Code::NotFound, "Feed not found")),
        }
    }

    pub async fn add_feed(
        mut app: MutexGuard<'_, App>,
        request: Request<AddFeedRequest>,
    ) -> Result<Response<AddFeedResponse>, Status> {
        let data = request.into_inner();
        let save = data.save();
        let feed = Feed::from(data.feed);
        let map = Arc::new(Mutex::new(app.feeds_map.clone()));
        let profiles = app.get_profiles().await;
        let client = app.nostr_service.get_client().await;
        let config = app.get_config().await;
        app.rss.feeds.push(feed.clone());

        let job = schedule(
            feed.schedule.clone().as_str(),
            feed.clone(),
            map,
            client,
            profiles,
            config,
        )
        .await;

        _ = app.rss.feeds_jobs.insert(feed.id.clone(), job.guid());
        _ = app.rss.scheduler.add(job).await;

        if save == true {
            _ = &app.update_feeds_config(&app.rss.feeds).await;
        }

        Ok(Response::new(AddFeedResponse {}))
    }

    // Interface to delete a feed on instance
    pub async fn delete_feed(
        mut app: MutexGuard<'_, App>,
        request: Request<DeleteFeedRequest>,
    ) -> Result<Response<DeleteFeedResponse>, Status> {
        let data = request.into_inner();

        let save = data.save();
        let feed_id = data.id;

        let idx = match app.rss.feeds.iter().position(|f| &f.id == &feed_id) {
            Some(idx) => idx,
            None => {
                return Err(Status::new(
                    Code::NotFound,
                    "No feed found with provided id",
                ));
            }
        };

        let app_clone = app.rss.clone();
        let job_uuid = app_clone.feeds_jobs.get(feed_id.trim());

        if job_uuid.is_none() {
            return Err(Status::new(
                Code::NotFound,
                "Job associated to feed not found",
            ));
        }

        _ = &app.rss.feeds.remove(idx);
        _ = &app.scheduler.remove(job_uuid.unwrap()).await;

        if save == true {
            _ = &app.update_feeds_config(&app.rss.feeds).await;
        }

        Ok(Response::new(grpc::DeleteFeedResponse {}))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::sync::Arc;

    use crate::grpc::grpctest_utils::mock_app;
    use nostrss_grpc::grpc::AddFeedRequest;
    use tokio::sync::Mutex;
    use tonic::Request;

    #[tokio::test]
    async fn feeds_list_test() {
        let app = Arc::new(Mutex::new(mock_app().await));

        let feeds_list_request = FeedsListRequest {};
        let request = Request::new(feeds_list_request);

        let feeds_list_request_result =
            FeedRequestHandler::feeds_list(app.lock().await, request).await;

        assert_eq!(feeds_list_request_result.is_ok(), true);

        let response = feeds_list_request_result.unwrap().into_inner();

        assert_eq!(response.feeds.len(), 3);
    }

    #[tokio::test]
    async fn add_feed_test() {
        let app = Arc::new(Mutex::new(mock_app().await));

        let add_feed_request = AddFeedRequest {
            feed: FeedItem {
                id: "test".to_string(),
                name: "my test feed".to_string(),
                url: "http://myrss.rs".to_string(),
                schedule: "1/10 * * * * *".to_string(),
                profiles: Vec::new(),
                tags: Vec::new(),
                template: None,
                cache_size: Some(50),
                pow_level: 50,
            },
            save: Some(false),
        };

        let request = Request::new(add_feed_request);

        let add_feed_result = {
            let app_lock = app.lock().await;
            FeedRequestHandler::add_feed(app_lock, request).await
        };

        assert_eq!(add_feed_result.is_ok(), true);

        let feeds_list_request = FeedsListRequest {};
        let request = Request::new(feeds_list_request);

        let feeds_list_request_result = {
            let app_lock = app.lock().await;
            FeedRequestHandler::feeds_list(app_lock, request).await
        };

        assert_eq!(feeds_list_request_result.is_ok(), true);

        let response = feeds_list_request_result.unwrap().into_inner();

        assert_eq!(response.feeds.len(), 4);
    }

    #[tokio::test]
    async fn delete_feed_test() {
        let app = Arc::new(Mutex::new(mock_app().await));

        let delete_feed_request = DeleteFeedRequest {
            id: "stackernews".to_string(),
            save: Some(false),
        };

        let request = Request::new(delete_feed_request);

        let delete_feed_request_result = {
            let app_lock = app.lock().await;
            FeedRequestHandler::delete_feed(app_lock, request).await
        };

        assert_eq!(delete_feed_request_result.is_ok(), true);

        let feeds_list_request = FeedsListRequest {};
        let request = Request::new(feeds_list_request);

        let feeds_list_request_result = {
            let app_lock = app.lock().await;
            FeedRequestHandler::feeds_list(app_lock, request).await
        };
        let response = feeds_list_request_result.unwrap().into_inner();

        assert_eq!(response.feeds.len(), 2);
    }

    #[tokio::test]
    async fn feed_info_test() {
        let app = Arc::new(Mutex::new(mock_app().await));

        let feed_info_request = FeedInfoRequest {
            id: "stackernews".to_string(),
        };
        let request = Request::new(feed_info_request);

        let feed_info_request_result =
            FeedRequestHandler::feed_info(app.lock().await, request).await;

        assert_eq!(feed_info_request_result.is_ok(), true);

        let response = feed_info_request_result.unwrap().into_inner();

        assert_eq!(response.feed.id, "stackernews");
    }
}
