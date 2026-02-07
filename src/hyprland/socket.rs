//! Hyprland Unix socket path resolution and raw I/O.

use super::error::HyprlandError;
use crate::config::HyprlandConfig;
use std::io::{BufReader, Read, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Resolves the Hyprland socket directory.
///
/// Priority:
/// 1. `config.socket_dir` (explicit override)
/// 2. `$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/`
///
/// # Errors
/// Returns error if environment variables are missing or directory doesn't exist.
pub fn resolve_socket_dir(config: &HyprlandConfig) -> Result<PathBuf, HyprlandError> {
    // Check config override first
    if let Some(ref dir) = config.socket_dir {
        let path = PathBuf::from(dir);
        if path.exists() {
            return Ok(path);
        }
        return Err(HyprlandError::SocketNotFound);
    }

    // Check instance signature override in config
    let instance_sig = if let Some(ref sig) = config.instance_signature {
        sig.clone()
    } else {
        std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
            .map_err(|_| HyprlandError::NoInstanceSignature)?
    };

    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").map_err(|_| HyprlandError::NoRuntimeDir)?;

    let socket_dir = PathBuf::from(runtime_dir).join("hypr").join(instance_sig);

    if socket_dir.exists() {
        Ok(socket_dir)
    } else {
        Err(HyprlandError::SocketNotFound)
    }
}

/// Returns the path to socket1 (command socket).
#[must_use]
pub fn socket1_path(socket_dir: &Path) -> PathBuf {
    socket_dir.join(".socket.sock")
}

/// Returns the path to socket2 (event socket).
#[must_use]
pub fn socket2_path(socket_dir: &Path) -> PathBuf {
    socket_dir.join(".socket2.sock")
}

/// Sends a command to Hyprland via socket1 and returns the response.
///
/// This is a short-lived connection: connect, write command, shutdown write half,
/// read full response, drop. Timeouts: 2s write, 5s read.
///
/// **Critical**: Leaving a socket1 connection open without closing the write half
/// will freeze Hyprland's IPC handler.
///
/// # Errors
/// Returns error on connection failure, timeout, or if Hyprland returns an error.
pub fn send_command(socket_dir: &Path, command: &str) -> Result<String, HyprlandError> {
    let path = socket1_path(socket_dir);
    let stream = UnixStream::connect(&path)?;

    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;

    // Write command
    let mut writer = &stream;
    writer.write_all(command.as_bytes())?;

    // Shutdown write half — signals Hyprland that the command is complete
    stream.shutdown(Shutdown::Write)?;

    // Read full response
    let mut response = String::new();
    let mut reader = &stream;
    reader.read_to_string(&mut response)?;

    Ok(response)
}

/// Connects to the Hyprland event stream (socket2).
///
/// Returns a buffered reader over a long-lived connection. The stream has a 1s
/// read timeout to allow periodic shutdown checks.
///
/// # Errors
/// Returns error on connection failure.
pub fn connect_event_stream(socket_dir: &Path) -> Result<BufReader<UnixStream>, HyprlandError> {
    let path = socket2_path(socket_dir);
    let stream = UnixStream::connect(&path)?;

    // 1s timeout allows periodic shutdown flag checks
    stream.set_read_timeout(Some(Duration::from_secs(1)))?;

    Ok(BufReader::new(stream))
}
