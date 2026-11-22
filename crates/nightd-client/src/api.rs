//! The types of the nightd HTTP API.

use serde::{Deserialize, Serialize};

/// The request to run an agent session.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RunRequest {
    /// The name of the agent to run.
    pub agent: String,
    /// The prompt the agent is started with.
    pub prompt: String,
}

/// The response of an agent session.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RunResponse {
    /// The agent's response.
    pub response: String,
}

/// The information about the nightd server.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InfoResponse {
    /// The current status of the nightd server.
    pub status: String,
}

/// The body of an API error response.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ErrorResponse {
    /// A human readable description of the error.
    pub error: String,
}
