//! Connection targets and their default values.

use std::{fmt, path::PathBuf};

/// The default address nightd binds to when TCP is used.
pub const DEFAULT_BIND: &str = "127.0.0.1";
/// The default port nightd listens on when TCP is used.
pub const DEFAULT_PORT: u16 = 8080;

/// Returns the default Unix socket path of nightd.
#[cfg(unix)]
pub fn default_socket_path() -> PathBuf {
    default_socket_dir().join("nightd.sock")
}

/// Resolves the Unix socket path, falling back to the default path.
#[cfg(unix)]
pub fn socket_path(path: Option<PathBuf>) -> PathBuf {
    path.unwrap_or_else(default_socket_path)
}

#[cfg(target_os = "linux")]
fn default_socket_dir() -> PathBuf {
    dirs::runtime_dir()
        .unwrap_or_else(|| PathBuf::from("/run"))
        .join("nightd")
}

#[cfg(all(unix, not(target_os = "linux")))]
fn default_socket_dir() -> PathBuf {
    std::env::temp_dir().join("nightd")
}

/// The target nightd server a client connects to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    /// Connect through a Unix socket.
    #[cfg(unix)]
    Unix {
        /// The path of the socket.
        path: PathBuf,
    },
    /// Connect through TCP.
    Tcp {
        /// The host of the server.
        host: String,
        /// The port of the server.
        port: u16,
    },
}

impl Target {
    /// Creates a target for a Unix socket.
    #[cfg(unix)]
    pub fn unix(path: impl Into<PathBuf>) -> Self {
        Self::Unix { path: path.into() }
    }

    /// Creates a target for a TCP host and port.
    pub fn tcp(host: impl Into<String>, port: u16) -> Self {
        Self::Tcp {
            host: host.into(),
            port,
        }
    }

    /// Resolves a target from optional connection settings.
    pub fn from_options(socket: Option<PathBuf>, host: Option<String>, port: Option<u16>) -> Self {
        if let Some(host) = host {
            return Self::tcp(host, port.unwrap_or(DEFAULT_PORT));
        }

        #[cfg(unix)]
        {
            Self::unix(socket_path(socket))
        }

        #[cfg(not(unix))]
        {
            let _ = socket;
            Self::tcp(DEFAULT_BIND, port.unwrap_or(DEFAULT_PORT))
        }
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(unix)]
            Self::Unix { path } => write!(f, "{} (unix)", path.display()),
            Self::Tcp { host, port } => write!(f, "{host}:{port}"),
        }
    }
}
