// Default to forbidding unsafe — only the ffi and hyprland modules legitimately need it for C-ABI and raw socket operations
#![cfg_attr(not(any(feature = "ffi", feature = "hyprland")), forbid(unsafe_code))]

//! `hyprlog` exists because Hyprland's ecosystem lacked a shared, structured
//! logging library — each tool rolled its own ad-hoc solution. This crate
//! provides a single, configurable logging layer that any Hypr project (or
//! external tool) can adopt, ensuring consistent log format, filtering, and
//! output across the ecosystem.
//!
//! Key design choices:
//! - Multiple output backends (terminal, file, JSON) so one logger serves dev, production, and tooling needs
//! - Customizable formatting templates to match Hyprland's existing log aesthetic
//! - Inline XML-like styling tags (`<bold>`, `<red>`) to avoid a separate markup dependency
//! - Builder pattern for ergonomic programmatic setup without TOML config files
//! - Interactive shell mode for ad-hoc log exploration and debugging sessions
//! - C-ABI FFI bindings so C/C++ Hyprland plugins can log through the same pipeline
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

// Always compiled regardless of feature flags — these form the minimum viable logger
pub mod cleanup;
pub mod config;
pub mod error;
pub mod fmt;
pub mod internal;
pub mod level;
pub mod logger;
pub mod output;

// Gated behind `cli` because library consumers typically don't need clap/rustyline dependencies
#[cfg(feature = "cli")]
pub mod cli;

// Interactive REPL shares the `cli` gate since it depends on rustyline and the CLI command set
#[cfg(feature = "cli")]
pub mod shell;

// Hyprland socket integration is optional — avoids pulling in Unix socket deps for non-Hyprland users
#[cfg(feature = "hyprland")]
pub mod hyprland;

// FFI requires unsafe and cbindgen — gated so pure-Rust consumers avoid that surface area
#[cfg(feature = "ffi")]
pub mod ffi;

// Flatten the most-used types to the crate root so callers can `use hyprlog::Logger` instead of `use hyprlog::logger::Logger`
pub use cleanup::{
    CleanupOptions, CleanupResult, LogFileInfo, LogStats, cleanup, format_size, parse_size, stats,
};
pub use config::Config;
pub use error::Error;
pub use fmt::{Alignment, Color, FormatValues, IconSet, IconType, TagConfig, Transform};
pub use level::Level;
pub use logger::{Logger, LoggerBuilder};
pub use output::{FileOutput, Output, TerminalOutput};

// Expose CLI helpers at crate root so downstream tools can reuse preset logic without deep imports
#[cfg(feature = "cli")]
pub use cli::PresetRunner;

// Surface Hyprland types at crate root for ergonomic access from event-monitoring tools
#[cfg(feature = "hyprland")]
pub use hyprland::{EventFormatter, EventListenerHandle, HyprlandEvent};

// Re-export every public FFI symbol so C consumers only need `#include "hyprlog.h"` with a single `-lhyprlog` link
#[cfg(feature = "ffi")]
pub use ffi::{
    HYPRLOG_LEVEL_DEBUG, HYPRLOG_LEVEL_ERROR, HYPRLOG_LEVEL_INFO, HYPRLOG_LEVEL_TRACE,
    HYPRLOG_LEVEL_WARN, HyprlogContext, hyprlog_debug, hyprlog_error, hyprlog_flush, hyprlog_free,
    hyprlog_get_last_error, hyprlog_info, hyprlog_init, hyprlog_init_simple, hyprlog_init_with_app,
    hyprlog_init_with_config, hyprlog_log, hyprlog_trace, hyprlog_warn,
};
