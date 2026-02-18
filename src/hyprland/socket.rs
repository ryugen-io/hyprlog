//! Hyprland Unix socket path resolution and event stream connection.

use crate::config::HyprlandConfig;
use crate::internal;
use hypr_sdk::ipc::instance;
use std::io::BufReader;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Resolves the Hyprland socket directory.
///
/// Priority:
/// 1. `config.socket_dir` (explicit override)
/// 2. `config.instance_signature`
/// 3. `hypr-sdk` current instance / discovery fallback
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

    let Some(instance_sig) = resolve_instance_signature(config) else {
        return None;
    };

    let socket_dir = PathBuf::from(instance::runtime_dir()).join(instance_sig);

    if socket_dir.exists() {
        Some(socket_dir)
    } else {
        internal::error("HYPRLAND", "Socket directory not found");
        None
    }
}

fn resolve_instance_signature(config: &HyprlandConfig) -> Option<String> {
    if let Some(sig) = &config.instance_signature {
        return Some(sig.clone());
    }

    if let Ok(current) = instance::current_instance() {
        return Some(current.signature);
    }

    if let Ok(instances) = instance::discover_instances()
        && let Some(first) = instances.first()
    {
        if instances.len() > 1 {
            internal::warn(
                "HYPRLAND",
                &format!(
                    "Multiple Hyprland instances found, using first discovered: {}",
                    first.signature
                ),
            );
        }
        return Some(first.signature.clone());
    }

    internal::error(
        "HYPRLAND",
        "Could not resolve Hyprland instance signature (set [hyprland].instance_signature)",
    );
    None
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
