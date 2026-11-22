//! A client and shared API types for the nightd HTTP API.

pub mod api;
mod client;
mod target;

pub use api::{ErrorResponse, InfoResponse, RunRequest, RunResponse};
pub use client::{Client, ClientError};
pub use target::{DEFAULT_BIND, DEFAULT_PORT, Target};
#[cfg(unix)]
pub use target::{default_socket_path, socket_path};
