//! The control plane of the nightd agent platform.

pub(crate) mod api;
pub(crate) mod cli;
pub(crate) mod server;

pub use cli::run;
#[cfg(unix)]
pub use server::default_socket_path;
pub use server::{Config, DEFAULT_BIND, DEFAULT_PORT, Server};
