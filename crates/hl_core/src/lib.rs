//! `hl_core` - Core logging library for hyprlog.
//!
//! A flexible, configurable logging library with support for:
//! - Multiple output backends (terminal, file)
//! - Customizable formatting templates
//! - Inline message styling with XML-like tags
//! - Builder pattern for programmatic configuration
//!
//! # Example
//!
//! ```
//! use hl_core::{Logger, Level};
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

pub mod cleanup;
pub mod color;
pub mod config;
pub mod format;
pub mod icon;
pub mod internal;
pub mod level;
pub mod logger;
pub mod output;
pub mod style;
pub mod tag;

// Re-exports for convenience
pub use cleanup::{
    CleanupOptions, CleanupResult, LogFileInfo, LogStats, cleanup, format_size, parse_size, stats,
};
pub use color::Color;
pub use config::Config;
pub use icon::{IconSet, IconType};
pub use level::Level;
pub use logger::{Logger, LoggerBuilder};
pub use output::{FileOutput, Output, OutputError, TerminalOutput};
pub use tag::{Alignment, TagConfig, Transform};
