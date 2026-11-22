//! The HTTP client for the nightd API.

use std::{error::Error, fmt};

use reqwest::StatusCode;

use crate::{
    api::{ErrorResponse, InfoResponse, RunRequest, RunResponse},
    target::Target,
};

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
            #[cfg(unix)]
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
