#[cfg(unix)]
use std::{
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

use nightd::{Config, Server};
use nightd_api::{
    InfoRequest, NightdServiceClient, RunRequest, Target, v1::info_response::Status as InfoStatus,
};
use tokio::task::JoinHandle;
use tonic::transport::Channel;
#[cfg(unix)]
use uuid::Uuid;

struct RunningServer {
    target: Target,
    #[cfg(unix)]
    socket: Option<PathBuf>,
    task: JoinHandle<()>,
}

impl RunningServer {
    #[cfg(unix)]
    async fn unix() -> Self {
        let socket = socket_path("api");
        let target = Target::unix(socket.clone());
        let server = Server::bind(Config::unix(&socket)).await.unwrap();
        let task = tokio::spawn(async move {
            server.serve().await.unwrap();
        });

        Self {
            target,
            socket: Some(socket),
            task,
        }
    }

    async fn tcp() -> Self {
        let server = Server::bind(Config::tcp([127, 0, 0, 1], 0)).await.unwrap();
        let address = server.address().unwrap();
        let port = address.rsplit_once(':').unwrap().1.parse().unwrap();
        let task = tokio::spawn(async move {
            server.serve().await.unwrap();
        });

        Self {
            target: Target::tcp("127.0.0.1", port),
            #[cfg(unix)]
            socket: None,
            task,
        }
    }

    async fn client(&self) -> NightdServiceClient<Channel> {
        nightd_api::client(&self.target).await.unwrap()
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
    let mut client = server.client().await;

    let info = client.info(InfoRequest {}).await.unwrap().into_inner();
    assert_eq!(info.status(), InfoStatus::Ok);

    let response = client
        .run(RunRequest {
            agent: "foo".to_owned(),
            prompt: "Hello".to_owned(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        response.response,
        "agent \"foo\" received prompt \"Hello\" (fake response)"
    );
}

#[tokio::test]
async fn tcp_server_serves_info_and_run() {
    let server = RunningServer::tcp().await;
    let mut client = server.client().await;

    let info = client.info(InfoRequest {}).await.unwrap().into_inner();
    assert_eq!(info.status(), InfoStatus::Ok);

    let response = client
        .run(RunRequest {
            agent: "foo".to_owned(),
            prompt: "Hello".to_owned(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        response.response,
        "agent \"foo\" received prompt \"Hello\" (fake response)"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn run_requires_agent_and_prompt() {
    let server = RunningServer::unix().await;
    let mut client = server.client().await;

    let cases = [
        (" ", "Hello", "agent is required"),
        ("foo", " ", "prompt is required"),
    ];

    for (agent, prompt, message) in cases {
        let status = client
            .run(RunRequest {
                agent: agent.to_owned(),
                prompt: prompt.to_owned(),
            })
            .await
            .unwrap_err();

        assert_eq!(status.code(), tonic::Code::InvalidArgument);
        assert_eq!(status.message(), message);
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
    let target = Target::unix(socket.clone());

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
        if let Ok(mut client) = nightd_api::client(&target).await
            && client.info(InfoRequest {}).await.is_ok()
        {
            ready = true;
            break;
        }

        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    assert!(ready, "nightd did not become ready");

    let mut client = nightd_api::client(&target).await.unwrap();
    let info = client.info(InfoRequest {}).await.unwrap().into_inner();
    assert_eq!(info.status(), InfoStatus::Ok);

    child.kill().unwrap();
    child.wait().unwrap();

    let _ = std::fs::remove_file(socket);
}
