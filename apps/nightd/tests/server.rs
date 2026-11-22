use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use nightd::{Config, InfoResponse, RunResponse, Server};
use reqwest::{Client, StatusCode};
use serde_json::json;
use tokio::task::JoinHandle;

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

struct RunningServer {
    address: String,
    socket: Option<PathBuf>,
    task: JoinHandle<()>,
}

impl RunningServer {
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
            socket: None,
            task,
        }
    }

    fn unix_client(&self) -> Client {
        unix_client(self.socket.as_deref().unwrap())
    }

    fn tcp_client(&self) -> Client {
        Client::new()
    }

    fn url(&self, path: &str) -> String {
        if self.socket.is_some() {
            format!("http://localhost{path}")
        } else {
            format!("http://{}{path}", self.address)
        }
    }
}

impl Drop for RunningServer {
    fn drop(&mut self) {
        self.task.abort();

        if let Some(socket) = &self.socket {
            let _ = std::fs::remove_file(socket);
        }
    }
}

fn socket_path(name: &str) -> PathBuf {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("nightd-{name}-{}-{id}.sock", std::process::id()))
}

fn unix_client(path: &Path) -> Client {
    Client::builder().unix_socket(path).build().unwrap()
}

#[tokio::test]
async fn unix_socket_serves_info_and_run() {
    let server = RunningServer::unix().await;
    let client = server.unix_client();

    let info: InfoResponse = client
        .get(server.url("/info"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(info.status, "OK");

    let response: RunResponse = client
        .post(server.url("/run"))
        .json(&json!({"agent": "foo", "prompt": "Hello"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        response.response,
        "agent \"foo\" received prompt \"Hello\" (fake response)"
    );
}

#[tokio::test]
async fn tcp_server_serves_info_and_run() {
    let server = RunningServer::tcp().await;
    let client = server.tcp_client();

    let info: InfoResponse = client
        .get(server.url("/info"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(info.status, "OK");

    let response: RunResponse = client
        .post(server.url("/run"))
        .json(&json!({"agent": "foo", "prompt": "Hello"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        response.response,
        "agent \"foo\" received prompt \"Hello\" (fake response)"
    );
}

#[tokio::test]
async fn run_requires_agent_and_prompt() {
    let server = RunningServer::unix().await;
    let client = server.unix_client();

    let cases = [
        (
            json!({"agent": " ", "prompt": "Hello"}),
            "agent is required",
        ),
        (json!({"agent": "foo", "prompt": " "}), "prompt is required"),
    ];

    for (request, message) in cases {
        let response = client
            .post(server.url("/run"))
            .json(&request)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response.text().await.unwrap();
        assert!(
            body.contains(message),
            "body {body:?} does not contain {message:?}"
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cli_starts_nightd_on_a_unix_socket() {
    let socket = socket_path("cli");
    let client = unix_client(&socket);

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
        if client
            .get("http://localhost/info")
            .send()
            .await
            .is_ok_and(|response| response.status().is_success())
        {
            ready = true;
            break;
        }

        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    assert!(ready, "nightd did not become ready");

    let response = client.get("http://localhost/info").send().await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body: InfoResponse = response.json().await.unwrap();
    assert_eq!(body.status, "OK");

    child.kill().unwrap();
    child.wait().unwrap();

    let _ = std::fs::remove_file(socket);
}
