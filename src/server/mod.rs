//! Remote logging server (feature: `rserver`).
//!
//! Programs on the same host or network send [`WireRecord`] JSON lines over a
//! Unix domain socket or TCP connection. The server dispatches each record
//! through a standard [`Logger`] with configurable outputs.
//!
//! # Daemon lifecycle
//!
//! ```bash
//! hyprlog server start    # fork background process, write PID file
//! hyprlog server status   # check if running
//! hyprlog server stop     # send SIGTERM via PID file
//! ```
//!
//! # Sending logs
//!
//! ```bash
//! hyprlog send info NET "connected"               # → Unix socket (default)
//! hyprlog send --tcp 127.0.0.1:9872 warn DB "slow query"
//! ```

pub mod config;
pub mod connection;
pub mod daemon;
pub mod listener;
pub mod protocol;

pub use config::ServerConfig;
pub use protocol::WireRecord;

use crate::internal;
use crate::level::Level;
use crate::logger::Logger;
use std::sync::Arc;

/// Runs the server in the foreground: writes PID, builds a [`Logger`] from
/// `config`, starts the Tokio runtime, and blocks until a shutdown signal.
///
/// The PID file is removed on exit regardless of how the server stops.
///
/// # Errors
/// Returns an error if the PID file cannot be written, the Tokio runtime
/// cannot be created, or either listener socket cannot be bound.
pub fn run(config: &ServerConfig) -> Result<(), crate::Error> {
    daemon::write_pid(config)?;

    let level: Level = config.log_level.parse().unwrap_or(Level::Info);
    let mut builder = Logger::builder().level(level);

    if config.terminal_enabled {
        builder = builder.terminal().colors(config.terminal_colors).done();
    }

    let logger = Arc::new(builder.build());

    internal::info("RSERVER", "starting hyprlog server");

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(crate::Error::Io)?;

    let result = rt.block_on(listener::run_listeners(config, logger));

    daemon::remove_pid(config);
    result
}
