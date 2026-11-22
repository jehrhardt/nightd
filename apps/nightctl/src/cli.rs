//! The command line interface of nightctl.

use std::error::Error;

use clap::{Args, Parser, Subcommand};
use nightd_api::{DEFAULT_BIND, DEFAULT_PORT, InfoRequest, RunRequest, client};

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
    /// Host of the nightd server
    #[arg(long, global = true, value_name = "HOST", default_value = DEFAULT_BIND, env = "NIGHTD_HOST")]
    host: String,

    /// Port of the nightd server
    #[arg(short, long, global = true, value_name = "PORT", default_value_t = DEFAULT_PORT, env = "NIGHTD_PORT")]
    port: u16,
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
    let host = cli.connection.host;
    let port = cli.connection.port;

    match cli.command {
        Command::Run { agent, prompt } => {
            let agent = agent.trim();
            if agent.is_empty() {
                return Err("--agent must not be empty".into());
            }

            let prompt = prompt.join(" ");
            let mut client = client(&host, port).await?;

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
            let mut client = client(&host, port).await?;

            let info = client
                .info(InfoRequest {})
                .await
                .map_err(|status| format!("fetch info from {host}:{port}: {status}"))?
                .into_inner();

            println!("status: {}", status_name(info.status()));
        }
    }

    Ok(())
}

fn status_name(status: nightd_api::v1::info_response::Status) -> &'static str {
    use nightd_api::v1::info_response::Status;

    match status {
        Status::Unspecified => "UNSPECIFIED",
        Status::Ok => "OK",
    }
}
