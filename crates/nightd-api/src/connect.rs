//! Connection helpers for the generated gRPC client.

use std::{error::Error, fmt};

use tonic::transport::{Channel, Endpoint};

use crate::NightdServiceClient;

/// An error returned while connecting to a nightd server.
#[derive(Debug)]
pub enum ConnectError {
    /// The endpoint URI is invalid.
    InvalidUri(String),
    /// The connection could not be established.
    Transport(tonic::transport::Error),
}

impl fmt::Display for ConnectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUri(error) => write!(f, "invalid endpoint: {error}"),
            Self::Transport(error) => write!(f, "{error}"),
        }
    }
}

impl Error for ConnectError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidUri(_) => None,
            Self::Transport(error) => Some(error),
        }
    }
}

impl From<tonic::transport::Error> for ConnectError {
    fn from(error: tonic::transport::Error) -> Self {
        Self::Transport(error)
    }
}

/// Connects to the nightd server at the given host and port.
pub async fn connect(host: &str, port: u16) -> Result<Channel, ConnectError> {
    let endpoint = Endpoint::from_shared(endpoint_url(host, port))
        .map_err(|error| ConnectError::InvalidUri(error.to_string()))?;

    endpoint.connect().await.map_err(ConnectError::Transport)
}

/// Creates a generated gRPC client connected to the nightd server.
pub async fn client(host: &str, port: u16) -> Result<NightdServiceClient<Channel>, ConnectError> {
    Ok(NightdServiceClient::new(connect(host, port).await?))
}

fn endpoint_url(host: &str, port: u16) -> String {
    if host.contains(':') && !host.starts_with('[') {
        format!("http://[{host}]:{port}")
    } else {
        format!("http://{host}:{port}")
    }
}
