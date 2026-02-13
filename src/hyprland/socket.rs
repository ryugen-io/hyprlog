//! Hyprland Unix socket path resolution and event stream connection.

use crate::config::HyprlandConfig;
use crate::internal;
use std::io::BufReader;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Resolves the Hyprland socket directory.
///
/// Priority:
/// 1. `config.socket_dir` (explicit override)
/// 2. `$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/`
///
/// Logs errors directly through hyprlog and returns `None` on failure.
#[must_use]
pub fn resolve_socket_dir(config: &HyprlandConfig) -> Option<PathBuf> {
    if let Some(ref dir) = config.socket_dir {
        let path = PathBuf::from(dir);
        if path.exists() {
            return Some(path);
        }
        internal::error("HYPRLAND", "Socket directory not found");
        return None;
    }

    let instance_sig = if let Some(ref sig) = config.instance_signature {
        sig.clone()
    } else if let Ok(sig) = std::env::var("HYPRLAND_INSTANCE_SIGNATURE") {
        sig
    } else {
        internal::error(
            "HYPRLAND",
            "HYPRLAND_INSTANCE_SIGNATURE not set (is Hyprland running?)",
        );
        return None;
    };

    let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") else {
        internal::error("HYPRLAND", "XDG_RUNTIME_DIR not set");
        return None;
    };

    let socket_dir = PathBuf::from(runtime_dir).join("hypr").join(instance_sig);

    if socket_dir.exists() {
        Some(socket_dir)
    } else {
        internal::error("HYPRLAND", "Socket directory not found");
        None
    }
}

/// Returns the path to socket2 (event socket).
#[must_use]
pub fn socket2_path(socket_dir: &Path) -> PathBuf {
    socket_dir.join(".socket2.sock")
}

/// Connects to the Hyprland event stream (socket2).
///
/// Logs errors directly through hyprlog and returns `None` on failure.
#[must_use]
pub fn connect_event_stream(socket_dir: &Path) -> Option<BufReader<UnixStream>> {
    let path = socket2_path(socket_dir);
    let stream = match UnixStream::connect(&path) {
        Ok(s) => s,
        Err(e) => {
            internal::error(
                "HYPRLAND",
                &format!("Failed to connect to event socket: {e}"),
            );
            return None;
        }
    };

    if let Err(e) = stream.set_read_timeout(Some(Duration::from_secs(1))) {
        internal::error("HYPRLAND", &format!("Failed to set socket timeout: {e}"));
        return None;
    }

    Some(BufReader::new(stream))
}
