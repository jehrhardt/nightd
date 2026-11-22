//! The nightd server.

use std::{fmt, io, net::IpAddr};

#[cfg(unix)]
use std::{
    os::unix::fs::{DirBuilderExt, FileTypeExt},
    path::{Path, PathBuf},
};

use tokio::net::TcpListener;
#[cfg(unix)]
use tokio::net::UnixListener;

use crate::api;

/// Configures how the nightd server listens for connections.
#[derive(Clone, Debug)]
pub enum Config {
    /// Listen on a Unix socket.
    #[cfg(unix)]
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
    #[cfg(unix)]
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
            #[cfg(unix)]
            Self::Unix { path } => write!(f, "{} (unix)", path.display()),
            Self::Tcp { bind, port } => write!(f, "{bind}:{port}"),
        }
    }
}

enum Listener {
    #[cfg(unix)]
    Unix(UnixListener),
    Tcp(TcpListener),
}

/// A bound nightd server that is ready to serve requests.
pub struct Server {
    listener: Listener,
    #[cfg(unix)]
    _socket: Option<SocketGuard>,
}

impl Server {
    /// Binds the server to the configured socket.
    pub async fn bind(config: Config) -> io::Result<Self> {
        match config {
            #[cfg(unix)]
            Config::Unix { path } => {
                ensure_socket_dir(&path)?;
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
                    #[cfg(unix)]
                    _socket: None,
                })
            }
        }
    }

    /// Returns the address the server is listening on.
    pub fn address(&self) -> io::Result<String> {
        match &self.listener {
            #[cfg(unix)]
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
        let app = api::router();

        #[cfg(unix)]
        let Self { listener, _socket } = self;
        #[cfg(not(unix))]
        let Self { listener } = self;

        match listener {
            #[cfg(unix)]
            Listener::Unix(listener) => axum::serve(listener, app).await,
            Listener::Tcp(listener) => axum::serve(listener, app).await,
        }?;

        Ok(())
    }
}

#[cfg(unix)]
fn ensure_socket_dir(path: &Path) -> io::Result<()> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };

    if parent.as_os_str().is_empty() {
        return Ok(());
    }

    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true).mode(0o700);
    builder.create(parent)
}

#[cfg(unix)]
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

#[cfg(unix)]
struct SocketGuard(PathBuf);

#[cfg(unix)]
impl Drop for SocketGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
