//! The generated nightd API and transport helpers.

mod connect;

/// The default address used by nightd and nightctl.
pub const DEFAULT_BIND: &str = "127.0.0.1";
/// The default port used by nightd and nightctl.
pub const DEFAULT_PORT: u16 = 8080;

/// The generated protobuf API.
pub mod v1 {
    tonic::include_proto!("nightd.v1");
}

pub use connect::{ConnectError, client, connect};
pub use v1::nightd_service_client::NightdServiceClient;
pub use v1::nightd_service_server::{NightdService, NightdServiceServer};
pub use v1::{InfoRequest, InfoResponse, RunRequest, RunResponse};
