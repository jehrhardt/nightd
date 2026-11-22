//! The control plane of the nightd agent platform.

use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    match nightd::run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("nightd: {error}");
            ExitCode::FAILURE
        }
    }
}
