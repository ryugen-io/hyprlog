//! Hyprland command and query wrappers over socket1.

use super::error::HyprlandError;
use super::socket;
use crate::config::HyprlandConfig;
use crate::internal;
use crate::logger::Logger;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

/// Sends a raw command/query to Hyprland and returns the response.
///
/// # Errors
/// Returns error on socket failure or if Hyprland is unreachable.
pub fn query(config: &HyprlandConfig, command: &str) -> Result<String, HyprlandError> {
    let socket_dir = socket::resolve_socket_dir(config)?;
    socket::send_command(&socket_dir, command)
}

/// Sends a query with the JSON flag prepended (`j/`).
///
/// # Errors
/// Returns error on socket failure.
pub fn query_json(config: &HyprlandConfig, command: &str) -> Result<String, HyprlandError> {
    let json_cmd = format!("j/{command}");
    query(config, &json_cmd)
}

/// Dispatches a Hyprland dispatcher command.
///
/// Sends `dispatch <args>` to Hyprland (e.g., `dispatch workspace 2`).
///
/// # Errors
/// Returns error on socket failure.
pub fn dispatch(config: &HyprlandConfig, args: &str) -> Result<String, HyprlandError> {
    let cmd = format!("dispatch {args}");
    query(config, &cmd)
}

/// Queries the Hyprland rolling log (one-shot).
///
/// # Errors
/// Returns error on socket failure.
pub fn rolling_log(config: &HyprlandConfig) -> Result<String, HyprlandError> {
    query(config, "rollinglog")
}

/// Follows the Hyprland rolling log, polling at ~500ms intervals.
///
/// Tracks the last-seen position and only logs new lines. Runs until the
/// shutdown flag is set.
pub fn follow_rolling_log(
    config: &HyprlandConfig,
    logger: &Logger,
    scope: &str,
    shutdown: &AtomicBool,
) {
    // Get initial snapshot to find starting position
    let mut last_len: usize = match rolling_log(config) {
        Ok(ref initial) => initial.len(),
        Err(e) => {
            internal::error(
                "HYPRLAND",
                &format!("Failed to get initial rolling log: {e}"),
            );
            return;
        }
    };

    while !shutdown.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(500));

        if shutdown.load(Ordering::Relaxed) {
            break;
        }

        match rolling_log(config) {
            Ok(log) => {
                if log.len() > last_len {
                    // New content appended
                    let new_content = &log[last_len..];
                    for line in new_content.lines() {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            logger.log(crate::level::Level::Info, scope, trimmed);
                        }
                    }
                    last_len = log.len();
                } else if log.len() < last_len {
                    // Log was rotated/reset — print everything
                    for line in log.lines() {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            logger.log(crate::level::Level::Info, scope, trimmed);
                        }
                    }
                    last_len = log.len();
                }
            }
            Err(e) => {
                internal::warn("HYPRLAND", &format!("Rolling log poll failed: {e}"));
            }
        }
    }
}
