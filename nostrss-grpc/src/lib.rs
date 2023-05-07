use grpc::nostrss_grpc_server::NostrssGrpc;
use grpc::StateRequest;
use grpc::StateResponse;
// use tonic::{transport::Server, Request, Response, Status};
use tonic;
// use nostrss_grpc::nostrss_grpc_server::NostrssGrpcServer;
use tonic::Request;
use tonic::Response;
use tonic::Status;
use tonic::Streaming;
pub mod grpc {
    include!("nostrss.rs");
}
