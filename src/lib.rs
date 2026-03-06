// Forbid unsafe code except when ffi or hyprland features are enabled
#![cfg_attr(not(any(feature = "ffi", feature = "hyprland")), forbid(unsafe_code))]

//! `hyprslog` - Flexible logging library for Hyprland and beyond.
//!
//! A configurable logging library with support for:
//! - Multiple output backends (terminal, file, JSON database)
//! - Customizable formatting templates
//! - Inline message styling with XML-like tags
//! - Builder pattern for programmatic configuration
//! - Interactive shell mode
//! - C-ABI FFI bindings
//!
//! # Example
//!
//! ```
//! use hyprs_log::{Logger, Level};
//!
//! let logger = Logger::builder()
//!     .level(Level::Debug)
//!     .terminal()
//!         .colors(true)
//!         .done()
//!     .build();
//!
//! logger.info("MAIN", "Application started");
//! logger.debug("NET", "Connecting to server...");
//! logger.warn("NET", "Connection timeout");
//! logger.error("NET", "Connection <bold>failed</bold>");
//! ```
//!
//! # Features
//!
//! - `cli` (default): Enables command-line interface and interactive shell
//! - `ffi`: Enables C-ABI FFI bindings

// Core modules (always available)
pub mod cleanup;
pub mod config;
pub mod error;
pub mod fmt;
pub mod internal;
pub mod level;
pub mod logger;
pub mod output;

// CLI module (feature-gated)
#[cfg(feature = "cli")]
pub mod cli;

// Shell module (feature-gated)
#[cfg(feature = "cli")]
pub mod shell;

// Hyprland IPC module (feature-gated)
#[cfg(feature = "hyprland")]
pub mod hyprland;

// Remote server module (feature-gated)
#[cfg(feature = "rserver")]
pub mod server;

// FFI module (feature-gated)
#[cfg(feature = "ffi")]
pub mod ffi;

// Re-exports for convenience
pub use cleanup::{
    CleanupOptions, CleanupResult, LogFileInfo, LogStats, cleanup, format_size, parse_size, stats,
};
pub use config::Config;
pub use error::Error;
pub use fmt::{Alignment, Color, FormatValues, IconSet, IconType, TagConfig, Transform};
pub use level::Level;
pub use logger::{Logger, LoggerBuilder};
pub use output::{FileOutput, Output, TerminalOutput};

// CLI re-exports
#[cfg(feature = "cli")]
pub use cli::PresetRunner;

// Hyprland re-exports
#[cfg(feature = "hyprland")]
pub use hyprland::{EventListenerHandle, HyprlandEvent};

// rserver re-exports
#[cfg(feature = "rserver")]
pub use logger::RemoteBuilder;
#[cfg(feature = "rserver")]
pub use output::RemoteOutput;
#[cfg(feature = "rserver")]
pub use server::ServerConfig;

// FFI re-exports
#[cfg(feature = "ffi")]
pub use ffi::{
    HYPRSLOG_LEVEL_DEBUG, HYPRSLOG_LEVEL_ERROR, HYPRSLOG_LEVEL_INFO, HYPRSLOG_LEVEL_TRACE,
    HYPRSLOG_LEVEL_WARN, HyprslogContext, hyprslog_debug, hyprslog_error, hyprslog_flush, hyprslog_free,
    hyprslog_get_last_error, hyprslog_info, hyprslog_init, hyprslog_init_simple, hyprslog_init_with_app,
    hyprslog_init_with_config, hyprslog_log, hyprslog_trace, hyprslog_warn,
};
