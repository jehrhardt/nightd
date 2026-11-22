//! The nightd server.

use std::{
    fmt, io,
    net::IpAddr,
    os::unix::fs::FileTypeExt,
    path::{Path, PathBuf},
};

use tokio::net::{TcpListener, UnixListener};

use crate::api;

/// The default address nightd binds to when TCP is used.
pub const DEFAULT_BIND: &str = "127.0.0.1";
/// The default port nightd listens on when TCP is used.
pub const DEFAULT_PORT: u16 = 8080;

/// Returns the default Unix socket path of nightd.
pub fn default_socket_path() -> PathBuf {
    std::env::temp_dir().join("nightd.sock")
}

/// Configures how the nightd server listens for connections.
#[derive(Clone, Debug)]
pub enum Config {
    /// Listen on a Unix socket.
    Unix {
        /// The path of the socket.
        path: PathBuf,
    },
    /// Listen on a TCP address.
    Tcp {
        /// The address to bind to.
        bind: IpAddr,
        /// The port to listen on. A port of 0 picks a free port.
        port: u16,
    },
}

impl Config {
    /// Creates a configuration for a Unix socket.
    pub fn unix(path: impl Into<PathBuf>) -> Self {
        Self::Unix { path: path.into() }
    }

    /// Creates a configuration for a TCP socket.
    pub fn tcp(bind: impl Into<IpAddr>, port: u16) -> Self {
        Self::Tcp {
            bind: bind.into(),
            port,
        }
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unix { path } => write!(f, "{} (unix)", path.display()),
            Self::Tcp { bind, port } => write!(f, "{bind}:{port}"),
        }
    }
}

enum Listener {
    Unix(UnixListener),
    Tcp(TcpListener),
}

/// A bound nightd server that is ready to serve requests.
pub struct Server {
    listener: Listener,
    _socket: Option<SocketGuard>,
}

impl Server {
    /// Binds the server to the configured socket.
    pub async fn bind(config: Config) -> io::Result<Self> {
        match config {
            Config::Unix { path } => {
                remove_stale_socket(&path)?;

                let listener = UnixListener::bind(&path)?;

                Ok(Self {
                    listener: Listener::Unix(listener),
                    _socket: Some(SocketGuard(path)),
                })
            }
            Config::Tcp { bind, port } => {
                let listener = TcpListener::bind((bind, port)).await?;

                Ok(Self {
                    listener: Listener::Tcp(listener),
                    _socket: None,
                })
            }
        }
    }

    /// Returns the address the server is listening on.
    pub fn address(&self) -> io::Result<String> {
        match &self.listener {
            Listener::Unix(listener) => listener.local_addr().map(|address| {
                address.as_pathname().map_or_else(
                    || "unix socket".to_owned(),
                    |path| path.display().to_string(),
                )
            }),
            Listener::Tcp(listener) => listener.local_addr().map(|address| address.to_string()),
        }
    }

    /// Serves requests until the server is stopped.
    pub async fn serve(self) -> io::Result<()> {
        let Self {
            listener,
            _socket: socket,
        } = self;

        let app = api::router();

        match listener {
            Listener::Unix(listener) => axum::serve(listener, app).await,
            Listener::Tcp(listener) => axum::serve(listener, app).await,
        }?;

        drop(socket);

        Ok(())
    }
}

fn remove_stale_socket(path: &Path) -> io::Result<()> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_socket() {
                std::fs::remove_file(path)
            } else {
                Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!("{} exists and is not a Unix socket", path.display()),
                ))
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

struct SocketGuard(PathBuf);

impl Drop for SocketGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
