//! The CLI to control nightd.

use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    match nightctl::run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("nightctl: {error}");
            ExitCode::FAILURE
        }
    }
}
