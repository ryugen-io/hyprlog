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
pub mod config;
pub mod fmt;
pub mod internal;
pub mod level;
pub mod logger;
pub mod output;

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
