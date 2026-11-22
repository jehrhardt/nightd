use std::{
    net::TcpListener,
    process::{Command, Stdio},
    time::Duration,
};

use nightd::Server;
use nightd_api::{
    InfoRequest, NightdServiceClient, RunRequest, v1::info_response::Status as InfoStatus,
};
use tokio::task::JoinHandle;
use tonic::transport::Channel;

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

    async fn client(&self) -> NightdServiceClient<Channel> {
        nightd_api::client(self.host, self.port).await.unwrap()
    }
}

impl Drop for RunningServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

fn free_port() -> u16 {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    listener.local_addr().unwrap().port()
}

#[tokio::test]
async fn tcp_server_serves_info_and_run() {
    let server = RunningServer::start().await;
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
async fn run_requires_agent_and_prompt() {
    let server = RunningServer::start().await;
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cli_starts_nightd_on_tcp() {
    let port = free_port();

    let mut child = Command::new(env!("CARGO_BIN_EXE_nightd"))
        .args(["start", "--bind", "127.0.0.1", "--port"])
        .arg(port.to_string())
        .env_remove("NIGHTD_HOST")
        .env_remove("NIGHTD_PORT")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let mut ready = false;
    for _ in 0..100 {
        if let Ok(mut client) = nightd_api::client("127.0.0.1", port).await
            && client.info(InfoRequest {}).await.is_ok()
        {
            ready = true;
            break;
        }

        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    assert!(ready, "nightd did not become ready");

    let mut client = nightd_api::client("127.0.0.1", port).await.unwrap();
    let info = client.info(InfoRequest {}).await.unwrap().into_inner();
    assert_eq!(info.status(), InfoStatus::Ok);

    child.kill().unwrap();
    child.wait().unwrap();
}
