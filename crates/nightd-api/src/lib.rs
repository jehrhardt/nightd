//! The generated nightd API and transport helpers.

mod connect;
mod target;

/// The generated protobuf API.
pub mod v1 {
    tonic::include_proto!("nightd.v1");
}

pub use connect::{ConnectError, client, connect};
pub use target::{DEFAULT_BIND, DEFAULT_PORT, Target};
#[cfg(unix)]
pub use target::{default_socket_path, socket_path};
pub use v1::nightd_service_client::NightdServiceClient;
pub use v1::nightd_service_server::{NightdService, NightdServiceServer};
pub use v1::{InfoRequest, InfoResponse, RunRequest, RunResponse};
