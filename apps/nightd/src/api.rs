//! The gRPC API exposed by nightd.

use nightd_api::{
    InfoRequest, InfoResponse, NightdService, NightdServiceServer, RunRequest, RunResponse,
    v1::info_response::Status as InfoStatus,
};
use tonic::{Request, Response, Status};

/// The nightd API service.
#[derive(Debug, Default)]
pub(crate) struct Api;

#[tonic::async_trait]
impl NightdService for Api {
    async fn run(&self, request: Request<RunRequest>) -> Result<Response<RunResponse>, Status> {
        let request = request.into_inner();

        let agent = request.agent.trim();
        if agent.is_empty() {
            return Err(Status::invalid_argument("agent is required"));
        }

        let prompt = request.prompt.trim();
        if prompt.is_empty() {
            return Err(Status::invalid_argument("prompt is required"));
        }

        Ok(Response::new(RunResponse {
            response: format!("agent {agent:?} received prompt {prompt:?} (fake response)"),
        }))
    }

    async fn info(&self, _request: Request<InfoRequest>) -> Result<Response<InfoResponse>, Status> {
        Ok(Response::new(InfoResponse {
            status: InfoStatus::Ok as i32,
        }))
    }
}

/// Creates the gRPC service.
pub(crate) fn service() -> NightdServiceServer<Api> {
    NightdServiceServer::new(Api)
}
