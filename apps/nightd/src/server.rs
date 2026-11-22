//! The nightd server.

use std::{io, net::IpAddr};

use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server as TonicServer;

use crate::api;

/// A bound nightd server that is ready to serve requests.
pub struct Server {
    listener: TcpListener,
}

impl Server {
    /// Binds the server to the configured TCP address.
    pub async fn bind(bind: impl Into<IpAddr>, port: u16) -> io::Result<Self> {
        let listener = TcpListener::bind((bind.into(), port)).await?;

        Ok(Self { listener })
    }

    /// Returns the address the server is listening on.
    pub fn address(&self) -> io::Result<String> {
        self.listener
            .local_addr()
            .map(|address| address.to_string())
    }

    /// Serves requests until the server is stopped.
    pub async fn serve(self) -> Result<(), tonic::transport::Error> {
        TonicServer::builder()
            .serve_with_incoming(api::service(), TcpListenerStream::new(self.listener))
            .await
    }
}
