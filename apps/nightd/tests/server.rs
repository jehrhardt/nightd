#[cfg(unix)]
use std::{
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

use nightd::{Config, Server};
use nightd_client::{Client, Target};
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
        let socket = socket_path("api");
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
    fn unix_client(&self) -> Client {
        Client::new(&Target::unix(self.socket.clone().unwrap())).unwrap()
    }

    fn tcp_client(&self) -> Client {
        let port = self.address.rsplit_once(':').unwrap().1.parse().unwrap();
        Client::new(&Target::tcp("127.0.0.1", port)).unwrap()
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

#[cfg(unix)]
#[tokio::test]
async fn unix_socket_serves_info_and_run() {
    let server = RunningServer::unix().await;
    let client = server.unix_client();

    let info = client.info().await.unwrap();
    assert_eq!(info.status, "OK");

    let response = client.run("foo", "Hello").await.unwrap();
    assert_eq!(
        response,
        "agent \"foo\" received prompt \"Hello\" (fake response)"
    );
}

#[tokio::test]
async fn tcp_server_serves_info_and_run() {
    let server = RunningServer::tcp().await;
    let client = server.tcp_client();

    let info = client.info().await.unwrap();
    assert_eq!(info.status, "OK");

    let response = client.run("foo", "Hello").await.unwrap();
    assert_eq!(
        response,
        "agent \"foo\" received prompt \"Hello\" (fake response)"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn run_requires_agent_and_prompt() {
    let server = RunningServer::unix().await;
    let client = server.unix_client();

    let cases = [
        (" ", "Hello", "agent is required"),
        ("foo", " ", "prompt is required"),
    ];

    for (agent, prompt, message) in cases {
        let error = client.run(agent, prompt).await.unwrap_err().to_string();

        assert!(
            error.contains(message),
            "error {error:?} does not contain {message:?}"
        );
    }
}

#[cfg(unix)]
#[tokio::test]
async fn creates_a_private_socket_directory() {
    use std::os::unix::fs::PermissionsExt;

    let socket = socket_path("mode")
        .with_extension("dir")
        .join("nightd.sock");
    let server = Server::bind(Config::unix(&socket)).await.unwrap();
    let dir = socket.parent().unwrap();

    let mode = std::fs::metadata(dir).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o700);

    drop(server);
    std::fs::remove_dir_all(dir).unwrap();
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cli_starts_nightd_on_a_unix_socket() {
    let socket = socket_path("cli");
    let client = Client::new(&Target::unix(socket.clone())).unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_nightd"))
        .arg("start")
        .arg("--socket")
        .arg(&socket)
        .env_remove("NIGHTD_SOCKET")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let mut ready = false;
    for _ in 0..100 {
        if client.info().await.is_ok() {
            ready = true;
            break;
        }

        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    assert!(ready, "nightd did not become ready");

    let info = client.info().await.unwrap();
    assert_eq!(info.status, "OK");

    child.kill().unwrap();
    child.wait().unwrap();

    let _ = std::fs::remove_file(socket);
}
