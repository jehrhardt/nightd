use std::process::{Command, Output};

use nightd::Server;
use tokio::task::JoinHandle;

struct RunningServer {
    host: &'static str,
    port: u16,
    task: JoinHandle<()>,
}

impl RunningServer {
    async fn start() -> Self {
        let host = "127.0.0.1";
        let server = Server::bind(host.parse::<std::net::IpAddr>().unwrap(), 0)
            .await
            .unwrap();
        let port = server
            .address()
            .unwrap()
            .rsplit_once(':')
            .unwrap()
            .1
            .parse()
            .unwrap();
        let task = tokio::spawn(async move {
            server.serve().await.unwrap();
        });

        Self { host, port, task }
    }
}

impl Drop for RunningServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

fn nightctl() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_nightctl"));
    command.env_remove("NIGHTD_HOST").env_remove("NIGHTD_PORT");
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn info_uses_host_and_port_from_the_environment() {
    let server = RunningServer::start().await;

    let output = output(
        nightctl()
            .arg("info")
            .env("NIGHTD_HOST", server.host)
            .env("NIGHTD_PORT", server.port.to_string()),
    );

    assert!(
        output.status.success(),
        "nightctl info failed: {}",
        combined(&output)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("status: OK"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn run_uses_host_and_port_flags() {
    let server = RunningServer::start().await;
    let port = server.port.to_string();

    let output = output(nightctl().args([
        "--host",
        server.host,
        "--port",
        port.as_str(),
        "run",
        "--agent",
        "foo",
        "Hello",
        "world",
    ]));

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
