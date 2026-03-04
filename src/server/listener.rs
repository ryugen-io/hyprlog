//! Tokio accept loops for Unix socket and TCP connections.

use crate::internal;
use crate::logger::Logger;
use crate::server::config::ServerConfig;
use crate::server::connection::handle_connection;
use std::sync::Arc;
use tokio::net::{TcpListener, UnixListener};

/// Binds both listeners and runs accept loops until a shutdown signal is received.
///
/// # Errors
/// Returns an error if either socket cannot be bound.
pub async fn run_listeners(config: &ServerConfig, logger: Arc<Logger>) -> Result<(), crate::Error> {
    // Remove a stale socket file left from a previous run.
    let _ = std::fs::remove_file(&config.socket_path);

    let unix = UnixListener::bind(&config.socket_path).map_err(crate::Error::Io)?;
    internal::info(
        "RSERVER",
        &format!("listening on unix:{}", config.socket_path),
    );

    let tcp = TcpListener::bind(config.tcp_addr())
        .await
        .map_err(crate::Error::Io)?;
    internal::info("RSERVER", &format!("listening on tcp:{}", config.tcp_addr()));

    let unix_log = Arc::clone(&logger);
    let tcp_log = Arc::clone(&logger);

    let unix_task = tokio::spawn(async move {
        loop {
            match unix.accept().await {
                Ok((stream, _)) => {
                    let log = Arc::clone(&unix_log);
                    tokio::spawn(handle_connection(stream, log));
                }
                Err(e) => internal::warn("RSERVER", &format!("unix accept error: {e}")),
            }
        }
    });

    let tcp_task = tokio::spawn(async move {
        loop {
            match tcp.accept().await {
                Ok((stream, addr)) => {
                    internal::trace("RSERVER", &format!("tcp connection from {addr}"));
                    let log = Arc::clone(&tcp_log);
                    tokio::spawn(handle_connection(stream, log));
                }
                Err(e) => internal::warn("RSERVER", &format!("tcp accept error: {e}")),
            }
        }
    });

    wait_for_shutdown().await;

    unix_task.abort();
    tcp_task.abort();
    internal::info("RSERVER", "server stopped");
    Ok(())
}

/// Waits for SIGTERM or SIGINT (Unix) / Ctrl-C (other platforms).
async fn wait_for_shutdown() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut sigterm = signal(SignalKind::terminate()).expect("SIGTERM handler");
        let mut sigint = signal(SignalKind::interrupt()).expect("SIGINT handler");
        tokio::select! {
            _ = sigterm.recv() => internal::info("RSERVER", "received SIGTERM"),
            _ = sigint.recv()  => internal::info("RSERVER", "received SIGINT"),
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}
