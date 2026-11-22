//! Connection helpers for the generated gRPC client.

use std::{error::Error, fmt};

#[cfg(unix)]
use tokio::net::UnixStream;
use tonic::transport::{Channel, Endpoint};
#[cfg(unix)]
use tower::service_fn;

use crate::{NightdServiceClient, Target};

/// An error returned while connecting to a nightd server.
#[derive(Debug)]
pub enum ConnectError {
    /// The TCP endpoint URI is invalid.
    InvalidUri(String),
    /// The connection could not be established.
    Transport(tonic::transport::Error),
    /// The Unix socket could not be opened.
    #[cfg(unix)]
    Io(std::io::Error),
}

impl fmt::Display for ConnectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUri(error) => write!(f, "invalid endpoint: {error}"),
            Self::Transport(error) => write!(f, "{error}"),
            #[cfg(unix)]
            Self::Io(error) => write!(f, "{error}"),
        }
    }
}

impl Error for ConnectError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidUri(_) => None,
            Self::Transport(error) => Some(error),
            #[cfg(unix)]
            Self::Io(error) => Some(error),
        }
    }
}

impl From<tonic::transport::Error> for ConnectError {
    fn from(error: tonic::transport::Error) -> Self {
        Self::Transport(error)
    }
}

#[cfg(unix)]
impl From<std::io::Error> for ConnectError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

/// Connects to the configured nightd server.
pub async fn connect(target: &Target) -> Result<Channel, ConnectError> {
    match target {
        #[cfg(unix)]
        Target::Unix { path } => {
            let path = path.clone();

            Endpoint::try_from("http://[::]:50051")
                .expect("the Unix socket endpoint URI is valid")
                .connect_with_connector(service_fn(move |_| {
                    let path = path.clone();
                    async move {
                        let stream = UnixStream::connect(path).await?;
                        Ok::<_, std::io::Error>(hyper_util::rt::TokioIo::new(stream))
                    }
                }))
                .await
                .map_err(ConnectError::Transport)
        }
        Target::Tcp { host, port } => {
            let url = tcp_url(host, *port);
            let endpoint = Endpoint::from_shared(url)
                .map_err(|error| ConnectError::InvalidUri(error.to_string()))?;

            endpoint.connect().await.map_err(ConnectError::Transport)
        }
    }
}

/// Creates a generated gRPC client connected to the configured server.
pub async fn client(target: &Target) -> Result<NightdServiceClient<Channel>, ConnectError> {
    Ok(NightdServiceClient::new(connect(target).await?))
}

fn tcp_url(host: &str, port: u16) -> String {
    if host.contains(':') && !host.starts_with('[') {
        format!("http://[{host}]:{port}")
    } else {
        format!("http://{host}:{port}")
    }
}
