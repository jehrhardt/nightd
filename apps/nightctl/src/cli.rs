//! The command line interface of nightctl.

use std::error::Error;

#[cfg(unix)]
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use nightd_api::{
    InfoRequest, RunRequest, Target, client, v1::info_response::Status as InfoStatus,
};

#[derive(Debug, Parser)]
#[command(
    name = "nightctl",
    version,
    about = "Control the nightd agent platform",
    long_about = "nightctl controls the nightd agent platform through its gRPC API."
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
    #[cfg(unix)]
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
        #[cfg(unix)]
        {
            Target::from_options(self.socket.clone(), self.host.clone(), self.port)
        }

        #[cfg(not(unix))]
        {
            Target::from_options(None, self.host.clone(), self.port)
        }
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
            let mut client = client(&target).await?;

            let response = client
                .run(RunRequest {
                    agent: agent.to_owned(),
                    prompt,
                })
                .await
                .map_err(|status| format!("run agent {agent:?}: {status}"))?
                .into_inner();

            println!("{}", response.response);
        }
        Command::Info => {
            let mut client = client(&target).await?;

            let info = client
                .info(InfoRequest {})
                .await
                .map_err(|status| format!("fetch info from {target}: {status}"))?
                .into_inner();

            println!("status: {}", status_name(info.status()));
        }
    }

    Ok(())
}

fn status_name(status: InfoStatus) -> &'static str {
    match status {
        InfoStatus::Unspecified => "UNSPECIFIED",
        InfoStatus::Ok => "OK",
    }
}
