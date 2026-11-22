//! The command line interface of nightd.

use std::{error::Error, net::IpAddr};

use clap::{Args, Parser, Subcommand};
use nightd_api::{DEFAULT_BIND, DEFAULT_PORT};

use crate::Server;

#[derive(Debug, Parser)]
#[command(
    name = "nightd",
    version,
    about = "Control plane for the nightd agent platform",
    long_about = "nightd is the control plane of the nightd agent platform and exposes its gRPC API."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Start the nightd server
    Start(StartArgs),
}

#[derive(Debug, Args)]
struct StartArgs {
    /// Address to bind the gRPC server to
    #[arg(long, value_name = "ADDR", default_value = DEFAULT_BIND)]
    bind: IpAddr,

    /// Port to listen on
    #[arg(short, long, value_name = "PORT", default_value_t = DEFAULT_PORT)]
    port: u16,
}

/// Runs nightd with the given process arguments.
pub async fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Command::Start(args) => start(args).await,
    }
}

async fn start(args: StartArgs) -> Result<(), Box<dyn Error>> {
    let server = Server::bind(args.bind, args.port).await?;

    println!("nightd listening on {}", server.address()?);

    tokio::select! {
        result = server.serve() => result?,
        () = shutdown_signal() => {}
    }

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            eprintln!("nightd: listen for interrupt: {error}");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(error) => {
                eprintln!("nightd: listen for terminate: {error}");
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {}
        () = terminate => {}
    }
}
