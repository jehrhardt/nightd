//! The HTTP API exposed by nightd.

use axum::{
    Json, Router,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use nightd_client::{ErrorResponse, InfoResponse, RunRequest, RunResponse};

/// An error returned by the API.
#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorResponse {
                error: self.message,
            }),
        )
            .into_response()
    }
}

/// Creates the router of the nightd API.
pub(crate) fn router() -> Router {
    Router::new()
        .route("/run", post(run))
        .route("/info", get(info))
}

async fn run(
    request: Result<Json<RunRequest>, JsonRejection>,
) -> Result<Json<RunResponse>, ApiError> {
    let Json(request) = request
        .map_err(|rejection| ApiError::new(StatusCode::BAD_REQUEST, rejection.body_text()))?;

    let agent = request.agent.trim();
    if agent.is_empty() {
        return Err(ApiError::bad_request("agent is required"));
    }

    let prompt = request.prompt.trim();
    if prompt.is_empty() {
        return Err(ApiError::bad_request("prompt is required"));
    }

    Ok(Json(RunResponse {
        response: format!("agent {agent:?} received prompt {prompt:?} (fake response)"),
    }))
}

async fn info() -> Json<InfoResponse> {
    Json(InfoResponse {
        status: "OK".to_owned(),
    })
}
