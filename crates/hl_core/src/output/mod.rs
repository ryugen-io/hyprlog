//! Output backends for log messages.

mod file;
mod terminal;

pub use file::FileOutput;
pub use terminal::TerminalOutput;

use crate::tag::TagConfig;
use crate::{Level, format::FormatValues};

/// A log record ready for output.
#[derive(Debug, Clone)]
pub struct LogRecord {
    pub level: Level,
    pub scope: String,
    pub message: String,
    pub values: FormatValues,
    /// Optional label override for custom display (e.g., "SUCCESS" instead of "INFO").
    pub label_override: Option<String>,
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
    fn write(&self, record: &LogRecord) -> Result<(), OutputError>;

    /// Flushes any buffered output.
    ///
    /// # Errors
    /// Returns an error if flushing fails.
    fn flush(&self) -> Result<(), OutputError>;
}

/// Error type for output operations.
#[derive(Debug)]
pub enum OutputError {
    /// I/O error.
    Io(std::io::Error),
    /// Format error.
    Format(String),
}

impl std::fmt::Display for OutputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Format(s) => write!(f, "format error: {s}"),
        }
    }
}

impl std::error::Error for OutputError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Format(_) => None,
        }
    }
}

impl From<std::io::Error> for OutputError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
