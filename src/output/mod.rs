//! The three built-in backends (terminal, file, JSON) can't cover every use case — the `Output`
//! trait lets users add custom backends without modifying hyprlog itself.

mod file;
mod json;
mod terminal;

pub use file::FileOutput;
pub use json::JsonOutput;
pub use terminal::TerminalOutput;

use crate::fmt::{FormatValues, TagConfig};
use crate::level::Level;

/// Carries all data a backend needs to render one log line — avoids passing a dozen loose parameters.
#[derive(Debug, Clone)]
pub struct LogRecord {
    pub level: Level,
    pub scope: String,
    pub message: String,
    pub values: FormatValues,
    /// Presets and custom events use domain-specific labels ("SUCCESS", "DEPLOY") that don't map to built-in levels.
    pub label_override: Option<String>,
    /// Presets can target a different app's log directory — `None` falls back to the logger's default.
    pub app_name: Option<String>,
    /// List items and continuation lines would look broken with repeated `[INFO] SCOPE` prefixes.
    pub raw: bool,
}

impl LogRecord {
    /// Backends shouldn't duplicate the label-override vs. default-level branching logic.
    #[must_use]
    pub fn format_tag(&self, tag_config: &TagConfig) -> String {
        self.label_override.as_ref().map_or_else(
            || tag_config.format(self.level),
            |label| tag_config.format_with_label(self.level, label),
        )
    }
}

/// `Send + Sync` bounds enable concurrent logging from multiple threads without locks on the trait object.
pub trait Output: Send + Sync {
    /// Each backend renders the record according to its own format (ANSI, plain text, JSON).
    ///
    /// # Errors
    /// I/O errors from the underlying sink (stderr, file, network).
    fn write(&self, record: &LogRecord) -> Result<(), crate::Error>;

    /// Buffered backends (file, JSON) may lose tail data on abrupt exit without an explicit flush.
    ///
    /// # Errors
    /// I/O errors from the underlying sink.
    fn flush(&self) -> Result<(), crate::Error>;
}
