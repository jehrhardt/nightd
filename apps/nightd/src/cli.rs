//! The command line interface of nightd.

use std::{error::Error, net::IpAddr, path::PathBuf};

use clap::{Args, Parser, Subcommand};

use crate::{Config, DEFAULT_PORT, Server, default_socket_path};

#[derive(Debug, Parser)]
#[command(
    name = "nightd",
    version,
    about = "Control plane for the nightd agent platform",
    long_about = "nightd is the control plane of the nightd agent platform and exposes its HTTP API."
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
    /// Path of the Unix socket to listen on (default when --bind is not set)
    #[arg(long, value_name = "PATH", env = "NIGHTD_SOCKET")]
    socket: Option<PathBuf>,

    /// Address to bind the TCP server to instead of a Unix socket
    #[arg(long, value_name = "ADDR")]
    bind: Option<IpAddr>,

    /// Port to listen on when --bind is set
    #[arg(short, long, value_name = "PORT", env = "NIGHTD_PORT")]
    port: Option<u16>,
}

/// Runs nightd with the given process arguments.
pub async fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Command::Start(args) => start(args).await,
    }
}

async fn start(args: StartArgs) -> Result<(), Box<dyn Error>> {
    let config = match args.bind {
        Some(bind) => Config::tcp(bind, args.port.unwrap_or(DEFAULT_PORT)),
        None => Config::Unix {
            path: args.socket.unwrap_or_else(default_socket_path),
        },
    };

    let server = Server::bind(config).await?;

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
