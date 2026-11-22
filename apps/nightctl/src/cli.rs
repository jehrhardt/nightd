//! The command line interface of nightctl.

use std::{error::Error, path::PathBuf};

use clap::{Args, Parser, Subcommand};

use crate::client::{self, Client, Target, default_socket_path};

#[derive(Debug, Parser)]
#[command(
    name = "nightctl",
    version,
    about = "Control the nightd agent platform",
    long_about = "nightctl controls the nightd agent platform through its HTTP API."
)]
struct Cli {
    #[command(flatten)]
    connection: ConnectionArgs,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Args)]
struct ConnectionArgs {
    /// Path of the Unix socket nightd listens on (default)
    #[arg(long, global = true, value_name = "PATH", env = "NIGHTD_SOCKET")]
    socket: Option<PathBuf>,

    /// Host of the nightd server to connect to over TCP
    #[arg(long, global = true, value_name = "HOST", env = "NIGHTD_HOST")]
    host: Option<String>,

    /// Port of the nightd server to connect to over TCP
    #[arg(short, long, global = true, value_name = "PORT", env = "NIGHTD_PORT")]
    port: Option<u16>,
}

impl ConnectionArgs {
    fn target(&self) -> Target {
        if let Some(host) = self.host.clone() {
            return Target::tcp(host, self.port.unwrap_or(client::DEFAULT_PORT));
        }

        Target::unix(self.socket.clone().unwrap_or_else(default_socket_path))
    }
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run an agent session and print the agent's response
    Run {
        /// Agent to run
        #[arg(long, value_name = "AGENT", required = true)]
        agent: String,

        /// Prompt to start the agent with
        #[arg(value_name = "PROMPT", required = true, num_args = 1..)]
        prompt: Vec<String>,
    },

    /// Show information about the nightd server
    Info,
}

/// Runs nightctl with the given process arguments.
pub async fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let target = cli.connection.target();

    match cli.command {
        Command::Run { agent, prompt } => {
            let agent = agent.trim();
            if agent.is_empty() {
                return Err("--agent must not be empty".into());
            }

            let prompt = prompt.join(" ");
            let client = Client::new(&target)?;

            let response = client
                .run(agent, &prompt)
                .await
                .map_err(|error| format!("run agent {agent:?}: {error}"))?;

            println!("{response}");
        }
        Command::Info => {
            let client = Client::new(&target)?;

            let info = client
                .info()
                .await
                .map_err(|error| format!("fetch info from {target}: {error}"))?;

            println!("status: {}", info.status);
        }
    }

    Ok(())
}
