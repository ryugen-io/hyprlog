//! Output backends for log messages.

mod file;
mod json;
mod terminal;

pub use file::FileOutput;
pub use json::JsonOutput;
pub use terminal::TerminalOutput;

use crate::fmt::{FormatValues, TagConfig};
use crate::level::Level;

/// A log record ready for output.
#[derive(Debug, Clone)]
pub struct LogRecord {
    pub level: Level,
    pub scope: String,
    pub message: String,
    pub values: FormatValues,
    /// Optional label override for custom display (e.g., "SUCCESS" instead of "INFO").
    pub label_override: Option<String>,
    /// Optional app name override (uses logger default if None).
    pub app_name: Option<String>,
    /// If true, output raw message without formatting (no tag, icon, scope).
    pub raw: bool,
}

impl LogRecord {
    /// Returns the formatted tag string, using `label_override` if set.
    #[must_use]
    pub fn format_tag(&self, tag_config: &TagConfig) -> String {
        self.label_override.as_ref().map_or_else(
            || tag_config.format(self.level),
            |label| tag_config.format_with_label(self.level, label),
        )
    }
}

/// Trait for log output backends.
pub trait Output: Send + Sync {
    /// Writes a log record.
    ///
    /// # Errors
    /// Returns an error if writing fails.
    fn write(&self, record: &LogRecord) -> Result<(), crate::Error>;

    /// Flushes any buffered output.
    ///
    /// # Errors
    /// Returns an error if flushing fails.
    fn flush(&self) -> Result<(), crate::Error>;
}
