//! CLI commands for the rserver feature.
//!
//! - `hyprlog server start` — fork daemon, write PID
//! - `hyprlog server stop`  — SIGTERM via PID file
//! - `hyprlog server status` — check if running
//! - `hyprlog server --foreground` — run in foreground (called internally by start)
//! - `hyprlog send [--app <a>] [--tcp <addr>] <level> <scope> <msg…>`

use crate::cli::util::parse_level;
use crate::internal;
use crate::server::config::ServerConfig;
use crate::server::daemon;
use crate::server::protocol::WireRecord;
use std::io::Write as _;
use std::net::TcpStream;
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt as _;
use std::process::{Child, ExitCode};
use std::time::Duration;

// ── `hyprlog server …` ────────────────────────────────────────────────────────

/// Dispatches `hyprlog server <subcommand>`.
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
                "usage: hyprlog server <start|stop|status>",
            );
            ExitCode::FAILURE
        }
    }
}

/// `hyprlog server start` — fork a background daemon.
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

/// `hyprlog server stop` — send SIGTERM via PID file.
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

/// `hyprlog server status` — print whether the server is running.
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
            println!("hyprlog server is running (PID {pid})");
            ExitCode::SUCCESS
        }
        _ => {
            println!("hyprlog server is not running");
            ExitCode::FAILURE
        }
    }
}

/// `hyprlog server --foreground` — run the server in the foreground.
///
/// This is called internally by `server start`.
fn server_foreground() -> ExitCode {
    let config = match ServerConfig::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("hyprlog-server: cannot load config: {e}");
            return ExitCode::FAILURE;
        }
    };

    match crate::server::run(&config) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("hyprlog-server: {e}");
            ExitCode::FAILURE
        }
    }
}

// ── `hyprlog send …` ─────────────────────────────────────────────────────────

/// `hyprlog send [--app <app>] [--tcp <addr>] <level> <scope> <msg…>`
///
/// Sends a single log record to the running server synchronously.
#[must_use]
pub fn cmd_send(args: &[&str]) -> ExitCode {
    let mut rest = args;
    let mut app: Option<String> = None;
    let mut tcp_addr: Option<String> = None;

    // Parse optional flags.
    loop {
        match rest {
            ["--app", a, tail @ ..] => {
                app = Some((*a).to_string());
                rest = tail;
            }
            ["--tcp", addr, tail @ ..] => {
                tcp_addr = Some((*addr).to_string());
                rest = tail;
            }
            _ => break,
        }
    }

    if rest.len() < 3 {
        internal::warn(
            "CLI",
            "usage: hyprlog send [--app <app>] [--tcp <addr>] <level> <scope> <message>",
        );
        return ExitCode::FAILURE;
    }

    let Some(level) = parse_level(rest[0]) else {
        internal::error("CLI", &format!("invalid level: {}", rest[0]));
        return ExitCode::FAILURE;
    };

    let scope = rest[1];
    let message = rest[2..].join(" ");

    let config = ServerConfig::load().unwrap_or_default();

    let wire = WireRecord::from_parts(level, scope, app.as_deref(), &message);
    let Ok(line) = wire.to_line() else {
        internal::error("CLI", "failed to serialize record");
        return ExitCode::FAILURE;
    };
    let bytes = line.as_bytes();

    if let Some(ref addr) = tcp_addr {
        match TcpStream::connect(addr) {
            Ok(mut s) => {
                if let Err(e) = s.write_all(bytes) {
                    internal::error("CLI", &format!("send failed: {e}"));
                    return ExitCode::FAILURE;
                }
            }
            Err(e) => {
                internal::error("CLI", &format!("cannot connect to {addr}: {e}"));
                return ExitCode::FAILURE;
            }
        }
    } else {
        match UnixStream::connect(&config.socket_path) {
            Ok(mut s) => {
                if let Err(e) = s.write_all(bytes) {
                    internal::error("CLI", &format!("send failed: {e}"));
                    return ExitCode::FAILURE;
                }
            }
            Err(e) => {
                internal::error(
                    "CLI",
                    &format!("cannot connect to {}: {e}", config.socket_path),
                );
                return ExitCode::FAILURE;
            }
        }
    }

    ExitCode::SUCCESS
}
