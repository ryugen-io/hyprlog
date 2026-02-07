// Forbid unsafe code except when ffi or hyprland features are enabled
#![cfg_attr(not(any(feature = "ffi", feature = "hyprland")), forbid(unsafe_code))]

//! `hyprlog` - Flexible logging library for Hyprland and beyond.
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
//! use hyprlog::{Logger, Level};
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

// FFI module (feature-gated)
#[cfg(feature = "ffi")]
pub mod ffi;

// Re-exports for convenience
pub use cleanup::{
    CleanupError, CleanupOptions, CleanupResult, LogFileInfo, LogStats, cleanup, format_size,
    parse_size, stats,
};
pub use config::Config;
pub use fmt::{Alignment, Color, FormatValues, IconSet, IconType, TagConfig, Transform};
pub use level::Level;
pub use logger::{Logger, LoggerBuilder};
pub use output::{FileOutput, Output, OutputError, TerminalOutput};

// CLI re-exports
#[cfg(feature = "cli")]
pub use cli::PresetRunner;

// Hyprland re-exports
#[cfg(feature = "hyprland")]
pub use hyprland::{EventListenerHandle, HyprlandError, HyprlandEvent};

// FFI re-exports
#[cfg(feature = "ffi")]
pub use ffi::{
    HYPRLOG_LEVEL_DEBUG, HYPRLOG_LEVEL_ERROR, HYPRLOG_LEVEL_INFO, HYPRLOG_LEVEL_TRACE,
    HYPRLOG_LEVEL_WARN, HyprlogContext, hyprlog_debug, hyprlog_error, hyprlog_flush, hyprlog_free,
    hyprlog_get_last_error, hyprlog_info, hyprlog_init, hyprlog_init_simple, hyprlog_init_with_app,
    hyprlog_init_with_config, hyprlog_log, hyprlog_trace, hyprlog_warn,
};
