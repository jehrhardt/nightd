//! The control plane of the nightd agent platform.

pub(crate) mod api;
pub(crate) mod cli;
pub(crate) mod server;

pub use api::{InfoResponse, RunRequest, RunResponse};
pub use cli::run;
pub use server::{Config, DEFAULT_BIND, DEFAULT_PORT, Server, default_socket_path};
