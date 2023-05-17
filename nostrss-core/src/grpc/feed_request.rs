use nostrss_grpc::grpc::{
    self, AddFeedRequest, AddFeedResponse, DeleteFeedRequest, DeleteFeedResponse, FeedInfoRequest,
    FeedInfoResponse, FeedItem, FeedsListRequest, FeedsListResponse,
};
use std::{ops::Index, sync::Arc};
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

    // Interface to delete a feed on instance
    pub async fn delete_feed(
        mut app: MutexGuard<'_, App>,
        request: Request<DeleteFeedRequest>,
    ) -> Result<Response<DeleteFeedResponse>, Status> {
        let feed_id = &request.into_inner().id;

        let idx = match app.rss.feeds.iter().position(|f| &f.id == feed_id) {
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
        _ = app.scheduler.remove(job_uuid.unwrap()).await;
        Ok(Response::new(grpc::DeleteFeedResponse {}))
    }

    pub async fn add_feed(
        mut app: MutexGuard<'_, App>,
        request: Request<AddFeedRequest>,
    ) -> Result<Response<AddFeedResponse>, Status> {
        let data = request.into_inner();
        let feed = Feed::from(data);
        let map = Arc::new(Mutex::new(app.feeds_map.clone()));
        let clients = Arc::new(Mutex::new(app.clients.clone()));

        app.rss.feeds.push(feed.clone());

        let job = schedule(feed.schedule.clone().as_str(), feed.clone(), map, clients).await;

        _ = app.rss.feeds_jobs.insert(feed.id.clone(), job.guid());
        _ = app.rss.scheduler.add(job).await;

        Ok(Response::new(AddFeedResponse {}))
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
            None => {
                return Err(Status::new(Code::NotFound, "Feed not found"));
            }
        }
    }
}
