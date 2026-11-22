//! The HTTP API exposed by nightd.

use axum::{
    Json, Router,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

/// The request to run an agent session.
#[derive(Debug, Deserialize, Serialize)]
pub struct RunRequest {
    /// The name of the agent to run.
    pub agent: String,
    /// The prompt the agent is started with.
    pub prompt: String,
}

/// The response of an agent session.
#[derive(Debug, Deserialize, Serialize)]
pub struct RunResponse {
    /// The agent's response.
    pub response: String,
}

/// The information about the nightd server.
#[derive(Debug, Deserialize, Serialize)]
pub struct InfoResponse {
    /// The current status of the nightd server.
    pub status: String,
}

/// The body of an API error response.
#[derive(Debug, Deserialize, Serialize)]
struct ErrorResponse {
    /// A human readable description of the error.
    pub error: String,
}

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
