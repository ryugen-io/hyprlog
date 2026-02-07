//! Error types for Hyprland IPC operations.

use std::fmt;
use std::io;

/// Error type for Hyprland IPC operations.
#[derive(Debug)]
pub enum HyprlandError {
    /// Hyprland socket directory not found.
    SocketNotFound,
    /// Failed to connect to Hyprland socket.
    ConnectionFailed(io::Error),
    /// `HYPRLAND_INSTANCE_SIGNATURE` environment variable not set.
    NoInstanceSignature,
    /// `XDG_RUNTIME_DIR` environment variable not set.
    NoRuntimeDir,
    /// Command returned an error response.
    CommandFailed(String),
    /// Failed to parse response.
    ParseError(String),
}

impl fmt::Display for HyprlandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SocketNotFound => write!(f, "Hyprland socket directory not found"),
            Self::ConnectionFailed(e) => write!(f, "failed to connect to Hyprland socket: {e}"),
            Self::NoInstanceSignature => {
                write!(
                    f,
                    "HYPRLAND_INSTANCE_SIGNATURE not set (is Hyprland running?)"
                )
            }
            Self::NoRuntimeDir => write!(f, "XDG_RUNTIME_DIR not set"),
            Self::CommandFailed(msg) => write!(f, "command failed: {msg}"),
            Self::ParseError(msg) => write!(f, "parse error: {msg}"),
        }
    }
}

impl std::error::Error for HyprlandError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ConnectionFailed(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for HyprlandError {
    fn from(e: io::Error) -> Self {
        Self::ConnectionFailed(e)
    }
}
