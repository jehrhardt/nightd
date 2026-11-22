//! The control plane of the nightd agent platform.

pub(crate) mod api;
pub(crate) mod cli;
pub(crate) mod server;

pub use cli::run;
pub use server::Server;
