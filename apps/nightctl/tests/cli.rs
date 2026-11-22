use std::process::{Command, Output};

#[cfg(unix)]
use std::path::{Path, PathBuf};

use nightd::{Config, Server};
use tokio::task::JoinHandle;
#[cfg(unix)]
use uuid::Uuid;

struct RunningServer {
    address: String,
    #[cfg(unix)]
    socket: Option<PathBuf>,
    task: JoinHandle<()>,
}

impl RunningServer {
    #[cfg(unix)]
    async fn unix() -> Self {
        let socket = socket_path("cli");
        let server = Server::bind(Config::unix(&socket)).await.unwrap();
        let address = server.address().unwrap();
        let task = tokio::spawn(async move {
            server.serve().await.unwrap();
        });

        Self {
            address,
            socket: Some(socket),
            task,
        }
    }

    async fn tcp() -> Self {
        let server = Server::bind(Config::tcp([127, 0, 0, 1], 0)).await.unwrap();
        let address = server.address().unwrap();
        let task = tokio::spawn(async move {
            server.serve().await.unwrap();
        });

        Self {
            address,
            #[cfg(unix)]
            socket: None,
            task,
        }
    }

    #[cfg(unix)]
    fn socket(&self) -> &Path {
        self.socket.as_deref().unwrap()
    }

    fn port(&self) -> u16 {
        self.address.rsplit_once(':').unwrap().1.parse().unwrap()
    }
}

impl Drop for RunningServer {
    fn drop(&mut self) {
        self.task.abort();

        #[cfg(unix)]
        if let Some(socket) = &self.socket {
            let _ = std::fs::remove_file(socket);
        }
    }
}

#[cfg(unix)]
fn socket_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("nightd-{name}-{}.sock", Uuid::new_v4()))
}

fn nightctl() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_nightctl"));
    command
        .env_remove("NIGHTD_SOCKET")
        .env_remove("NIGHTD_HOST")
        .env_remove("NIGHTD_PORT");
    command
}

fn output(command: &mut Command) -> Output {
    command.output().unwrap()
}

fn combined(output: &Output) -> String {
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    text
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn info_uses_the_socket_from_the_environment() {
    let server = RunningServer::unix().await;

    let output = output(nightctl().arg("info").env("NIGHTD_SOCKET", server.socket()));

    assert!(
        output.status.success(),
        "nightctl info failed: {}",
        combined(&output)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("status: OK"));
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn run_uses_the_socket_flag() {
    let server = RunningServer::unix().await;

    let output = output(
        nightctl()
            .args(["--socket"])
            .arg(server.socket())
            .args(["run", "--agent", "foo", "Hello", "world"]),
    );

    assert!(
        output.status.success(),
        "nightctl run failed: {}",
        combined(&output)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "agent \"foo\" received prompt \"Hello world\" (fake response)"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn info_and_run_over_tcp() {
    let server = RunningServer::tcp().await;
    let port = server.port().to_string();

    let info = output(nightctl().args(["--host", "127.0.0.1", "--port", port.as_str(), "info"]));

    assert!(
        info.status.success(),
        "nightctl info failed: {}",
        combined(&info)
    );
    assert!(String::from_utf8_lossy(&info.stdout).contains("status: OK"));

    let run = output(nightctl().args([
        "--host",
        "127.0.0.1",
        "--port",
        port.as_str(),
        "run",
        "--agent",
        "foo",
        "Hello",
    ]));

    assert!(
        run.status.success(),
        "nightctl run failed: {}",
        combined(&run)
    );
    assert!(
        String::from_utf8_lossy(&run.stdout).contains("agent \"foo\" received prompt \"Hello\"")
    );
}

#[tokio::test]
async fn run_requires_an_agent() {
    let output = output(nightctl().args(["run", "Hello"]));

    assert!(!output.status.success());
    assert!(
        combined(&output).contains("--agent"),
        "output does not mention --agent: {}",
        combined(&output)
    );
}

#[tokio::test]
async fn run_rejects_an_empty_agent() {
    let output = output(nightctl().args(["run", "--agent", "", "Hello"]));

    assert!(!output.status.success());
    assert!(
        combined(&output).contains("--agent must not be empty"),
        "output does not mention the empty agent: {}",
        combined(&output)
    );
}
