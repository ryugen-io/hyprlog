//! CLI commands for the rserver feature.
//!
//! - `hyprslog server start` — fork daemon, write PID
//! - `hyprslog server stop`  — SIGTERM via PID file
//! - `hyprslog server status` — check if running
//! - `hyprslog server --foreground` — run in foreground (called internally by start)

use crate::internal;
use crate::server::config::ServerConfig;
use crate::server::daemon;
use std::os::unix::process::CommandExt as _;
use std::process::{Child, ExitCode};
use std::time::Duration;

// ── `hyprslog server …` ────────────────────────────────────────────────────────

/// Dispatches `hyprslog server <subcommand>`.
#[must_use]
pub fn cmd_server(args: &[&str]) -> ExitCode {
    match args.first().copied() {
        Some("start") => server_start(),
        Some("stop") => server_stop(),
        Some("status") => server_status(),
        Some("--foreground") => server_foreground(),
        _ => {
            internal::warn(
                "CLI",
                "usage: hyprslog server <start|stop|status>",
            );
            ExitCode::FAILURE
        }
    }
}

/// `hyprslog server start` — fork a background daemon.
fn server_start() -> ExitCode {
    let config = match ServerConfig::load() {
        Ok(c) => c,
        Err(e) => {
            internal::error("CLI", &format!("cannot load server config: {e}"));
            return ExitCode::FAILURE;
        }
    };

    // Bail if already running.
    if let Ok(Some(pid)) = daemon::read_pid(&config)
        && daemon::pid_is_running(pid)
    {
        internal::error("CLI", &format!("server already running (PID {pid})"));
        return ExitCode::FAILURE;
    }

    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            internal::error("CLI", &format!("cannot locate binary: {e}"));
            return ExitCode::FAILURE;
        }
    };

    let mut child: Child = match std::process::Command::new(&exe)
        .args(["server", "--foreground"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .process_group(0) // detach from the parent's process group
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            internal::error("CLI", &format!("failed to spawn server: {e}"));
            return ExitCode::FAILURE;
        }
    };

    // Give the daemon a moment to start.
    std::thread::sleep(Duration::from_millis(200));

    if let Ok(Some(status)) = child.try_wait() {
        internal::error("CLI", &format!("server exited immediately: {status}"));
        ExitCode::FAILURE
    } else {
        internal::info("CLI", &format!("server started (PID {})", child.id()));
        ExitCode::SUCCESS
    }
}

/// `hyprslog server stop` — send SIGTERM via PID file.
fn server_stop() -> ExitCode {
    let config = match ServerConfig::load() {
        Ok(c) => c,
        Err(e) => {
            internal::error("CLI", &format!("cannot load server config: {e}"));
            return ExitCode::FAILURE;
        }
    };

    match daemon::read_pid(&config) {
        Ok(Some(pid)) if daemon::pid_is_running(pid) => {
            match daemon::send_sigterm(pid) {
                Ok(()) => {
                    internal::info("CLI", &format!("SIGTERM sent to PID {pid}"));
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    internal::error("CLI", &format!("kill failed: {e}"));
                    ExitCode::FAILURE
                }
            }
        }
        _ => {
            internal::warn("CLI", "server is not running");
            ExitCode::FAILURE
        }
    }
}

/// `hyprslog server status` — print whether the server is running.
fn server_status() -> ExitCode {
    let config = match ServerConfig::load() {
        Ok(c) => c,
        Err(e) => {
            internal::error("CLI", &format!("cannot load server config: {e}"));
            return ExitCode::FAILURE;
        }
    };

    match daemon::read_pid(&config) {
        Ok(Some(pid)) if daemon::pid_is_running(pid) => {
            println!("hyprslog server is running (PID {pid})");
            ExitCode::SUCCESS
        }
        _ => {
            println!("hyprslog server is not running");
            ExitCode::FAILURE
        }
    }
}

/// `hyprslog server --foreground` — run the server in the foreground.
///
/// This is called internally by `server start`.
fn server_foreground() -> ExitCode {
    let config = match ServerConfig::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("hyprslog-server: cannot load config: {e}");
            return ExitCode::FAILURE;
        }
    };

    match crate::server::run(&config) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("hyprslog-server: {e}");
            ExitCode::FAILURE
        }
    }
}


