//! A client for the nightd HTTP API.

use std::{error::Error, fmt, path::PathBuf};

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

/// The target nightd server the client connects to.
#[derive(Clone, Debug)]
pub enum Target {
    /// Connect through a Unix socket.
    Unix {
        /// The path of the socket.
        path: PathBuf,
    },
    /// Connect through TCP.
    Tcp {
        /// The host of the server.
        host: String,
        /// The port of the server.
        port: u16,
    },
}

impl Target {
    /// Creates a target for a Unix socket.
    pub fn unix(path: impl Into<PathBuf>) -> Self {
        Self::Unix { path: path.into() }
    }

    /// Creates a target for a TCP host and port.
    pub fn tcp(host: impl Into<String>, port: u16) -> Self {
        Self::Tcp {
            host: host.into(),
            port,
        }
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unix { path } => write!(f, "{} (unix)", path.display()),
            Self::Tcp { host, port } => write!(f, "{host}:{port}"),
        }
    }
}

/// A client for the nightd HTTP API.
pub struct Client {
    client: reqwest::Client,
    base_url: String,
}

impl Client {
    /// Creates a client for the given target.
    pub fn new(target: &Target) -> Result<Self, reqwest::Error> {
        let builder = reqwest::Client::builder();

        let (builder, base_url) = match target {
            Target::Unix { path } => (
                builder.unix_socket(path.as_path()),
                "http://localhost".to_owned(),
            ),
            Target::Tcp { host, port } => (builder, tcp_base_url(host, *port)),
        };

        Ok(Self {
            client: builder.build()?,
            base_url,
        })
    }

    /// Starts an agent session and returns the agent's response.
    pub async fn run(&self, agent: &str, prompt: &str) -> Result<String, ClientError> {
        let response = self
            .client
            .post(format!("{}/run", self.base_url))
            .json(&RunRequest {
                agent: agent.to_owned(),
                prompt: prompt.to_owned(),
            })
            .send()
            .await?;

        let response = self.successful(response).await?;

        Ok(response.json::<RunResponse>().await?.response)
    }

    /// Fetches information about the nightd server.
    pub async fn info(&self) -> Result<InfoResponse, ClientError> {
        let response = self
            .client
            .get(format!("{}/info", self.base_url))
            .send()
            .await?;

        let response = self.successful(response).await?;

        Ok(response.json::<InfoResponse>().await?)
    }

    async fn successful(
        &self,
        response: reqwest::Response,
    ) -> Result<reqwest::Response, ClientError> {
        let status = response.status();

        if status.is_success() {
            return Ok(response);
        }

        let body = response.text().await?;
        let message = serde_json::from_str::<ErrorResponse>(&body)
            .map(|response| response.error)
            .unwrap_or_else(|_| body.trim().to_owned());

        Err(ClientError::Api { status, message })
    }
}

fn tcp_base_url(host: &str, port: u16) -> String {
    if host.contains(':') && !host.starts_with('[') {
        format!("http://[{host}]:{port}")
    } else {
        format!("http://{host}:{port}")
    }
}

#[derive(Debug, Serialize)]
struct RunRequest {
    agent: String,
    prompt: String,
}

#[derive(Debug, Deserialize)]
struct RunResponse {
    response: String,
}

/// The information about the nightd server.
#[derive(Debug, Deserialize)]
pub struct InfoResponse {
    /// The current status of the nightd server.
    pub status: String,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: String,
}

/// An error returned by the nightd client.
#[derive(Debug)]
pub enum ClientError {
    /// The request could not be sent or the response could not be read.
    Http(reqwest::Error),
    /// The server responded with an error.
    Api {
        /// The HTTP status of the response.
        status: StatusCode,
        /// The error message of the response.
        message: String,
    },
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http(error) => write!(f, "{error}"),
            Self::Api { status, message } if message.is_empty() => {
                write!(f, "server returned {status}")
            }
            Self::Api { status, message } => write!(f, "server returned {status}: {message}"),
        }
    }
}

impl Error for ClientError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Http(error) => Some(error),
            Self::Api { .. } => None,
        }
    }
}

impl From<reqwest::Error> for ClientError {
    fn from(error: reqwest::Error) -> Self {
        Self::Http(error)
    }
}

/// Returns the default Unix socket path of nightd.
pub fn default_socket_path() -> PathBuf {
    std::env::temp_dir().join("nightd.sock")
}

/// Returns the default port nightd listens on when TCP is used.
pub const DEFAULT_PORT: u16 = 8080;

/// Returns the target configured by the environment.
pub fn target_from_env() -> Target {
    match std::env::var("NIGHTD_HOST") {
        Ok(host) if !host.is_empty() => Target::tcp(
            host,
            std::env::var("NIGHTD_PORT")
                .ok()
                .and_then(|port| port.parse().ok())
                .unwrap_or(DEFAULT_PORT),
        ),
        _ => Target::unix(
            std::env::var_os("NIGHTD_SOCKET")
                .map(PathBuf::from)
                .unwrap_or_else(default_socket_path),
        ),
    }
}
