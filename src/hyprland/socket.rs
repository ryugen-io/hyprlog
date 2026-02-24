//! Hyprland can run multiple instances with different runtime directories —
//! finding the right socket requires a priority chain of config overrides,
//! instance signatures, and discovery fallbacks.

use crate::config::HyprlandConfig;
use crate::internal;
use hypr_sdk::ipc::instance;
use std::path::{Path, PathBuf};

/// Resolve the Hyprland runtime socket directory.
///
/// Explicit overrides take priority because containers and custom setups may not
/// follow standard runtime directory conventions. The fallback chain keeps
/// zero-config behavior for the common single-instance case while still
/// supporting multi-instance setups.
#[must_use]
pub fn resolve_socket_dir(config: &HyprlandConfig) -> Option<PathBuf> {
    let scope = config.scope.as_str();
    if let Some(ref dir) = config.socket_dir {
        let path = PathBuf::from(dir);
        if path.exists() {
            return Some(path);
        }
        internal::error(scope, "Socket directory not found");
        return None;
    }

    let instance_sig = resolve_instance_signature(config)?;

    let socket_dir = PathBuf::from(instance::runtime_dir()).join(instance_sig);

    if socket_dir.exists() {
        Some(socket_dir)
    } else {
        internal::error(scope, "Socket directory not found");
        None
    }
}

fn resolve_instance_signature(config: &HyprlandConfig) -> Option<String> {
    let scope = config.scope.as_str();
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
                scope,
                &format!(
                    "Multiple Hyprland instances found, using first discovered: {}",
                    first.signature
                ),
            );
        }
        return Some(first.signature.clone());
    }

    internal::error(
        scope,
        "Could not resolve Hyprland instance signature (set [hyprland].instance_signature)",
    );
    None
}

/// Socket2 is the event stream socket — socket1 is for commands, which hyprlog doesn't need.
#[must_use]
pub fn socket2_path(socket_dir: &Path) -> PathBuf {
    socket_dir.join(".socket2.sock")
}
