//! `hyprslog send [--app <app>] [--tcp <addr>] <level> <scope> <msg…>`
//!
//! Sends a single log record to a running hyprslog server over Unix socket or TCP.
//! No `rserver` feature required — just std + `serde_json`.

use crate::cli::util::parse_level;
use crate::internal;
use std::io::Write as _;
use std::net::TcpStream;
use std::os::unix::net::UnixStream;
use std::process::ExitCode;

fn default_socket_path() -> String {
    std::env::var("XDG_RUNTIME_DIR")
        .map_or_else(|_| "/tmp/hyprslog.sock".to_string(), |r| format!("{r}/hyprslog.sock"))
}

/// `hyprslog send [--app <app>] [--tcp <addr>] <level> <scope> <msg…>`
#[must_use]
pub fn cmd_send(args: &[&str]) -> ExitCode {
    let mut rest = args;
    let mut app: Option<&str> = None;
    let mut tcp_addr: Option<&str> = None;

    loop {
        match rest {
            ["--app", a, tail @ ..] => {
                app = Some(a);
                rest = tail;
            }
            ["--tcp", addr, tail @ ..] => {
                tcp_addr = Some(addr);
                rest = tail;
            }
            _ => break,
        }
    }

    if rest.len() < 3 {
        internal::warn(
            "CLI",
            "usage: hyprslog send [--app <app>] [--tcp <addr>] <level> <scope> <message>",
        );
        return ExitCode::FAILURE;
    }

    let Some(level) = parse_level(rest[0]) else {
        internal::error("CLI", &format!("invalid level: {}", rest[0]));
        return ExitCode::FAILURE;
    };

    let scope = rest[1];
    let message = rest[2..].join(" ");

    let line = format!(
        "{}\n",
        serde_json::json!({
            "level":   level.as_str(),
            "scope":   scope,
            "app":     app,
            "message": message,
        })
    );
    let bytes = line.as_bytes();

    if let Some(addr) = tcp_addr {
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
        let path = default_socket_path();
        match UnixStream::connect(&path) {
            Ok(mut s) => {
                if let Err(e) = s.write_all(bytes) {
                    internal::error("CLI", &format!("send failed: {e}"));
                    return ExitCode::FAILURE;
                }
            }
            Err(e) => {
                internal::error("CLI", &format!("cannot connect to {path}: {e}"));
                return ExitCode::FAILURE;
            }
        }
    }

    ExitCode::SUCCESS
}
