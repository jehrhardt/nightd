//! The command line interface of nightd.

use std::{error::Error, net::IpAddr};

#[cfg(unix)]
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[cfg(not(unix))]
use nightd_client::DEFAULT_BIND;
use nightd_client::DEFAULT_PORT;
#[cfg(unix)]
use nightd_client::socket_path;

use crate::{Config, Server};

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
    #[cfg(unix)]
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
        None => default_config(args),
    };

    let server = Server::bind(config).await?;

    println!("nightd listening on {}", server.address()?);

    tokio::select! {
        result = server.serve() => result?,
        () = shutdown_signal() => {}
    }

    Ok(())
}

#[cfg(unix)]
fn default_config(args: StartArgs) -> Config {
    Config::Unix {
        path: socket_path(args.socket),
    }
}

#[cfg(not(unix))]
fn default_config(_args: StartArgs) -> Config {
    Config::tcp(
        DEFAULT_BIND
            .parse::<IpAddr>()
            .expect("the default bind address is valid"),
        DEFAULT_PORT,
    )
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
