//! Main logger struct with builder pattern.

mod builder;
mod from_config;
mod json_builder;

pub use builder::{FileBuilder, LoggerBuilder, TerminalBuilder};
pub use json_builder::JsonBuilder;

use crate::config::PresetConfig;
use crate::fmt::FormatValues;
use crate::internal;
use crate::level::Level;
use crate::output::{LogRecord, Output, OutputError};
use std::collections::HashMap;

/// The main logger.
#[derive(Default)]
pub struct Logger {
    min_level: Level,
    outputs: Vec<Box<dyn Output>>,
    presets: HashMap<String, PresetConfig>,
    pub(crate) app_name: Option<String>,
}

impl Logger {
    /// Creates a new logger builder.
    #[must_use]
    pub fn builder() -> LoggerBuilder {
        LoggerBuilder::new()
    }

    /// Logs a message at the given level.
    pub fn log(&self, level: Level, scope: &str, msg: &str) {
        if level < self.min_level {
            return;
        }

        let record = LogRecord {
            level,
            scope: scope.to_string(),
            message: msg.to_string(),
            values: FormatValues::new(),
            label_override: None,
            app_name: self.app_name.clone(),
            raw: false,
        };

        for output in &self.outputs {
            let _ = output.write(&record);
        }
    }

    /// Logs a message with a custom label override.
    pub fn log_with_label(&self, level: Level, scope: &str, msg: &str, label: &str) {
        if level < self.min_level {
            return;
        }

        let record = LogRecord {
            level,
            scope: scope.to_string(),
            message: msg.to_string(),
            values: FormatValues::new(),
            label_override: Some(label.to_string()),
            app_name: self.app_name.clone(),
            raw: false,
        };

        for output in &self.outputs {
            let _ = output.write(&record);
        }
    }

    /// Logs a message with full control options, including app name override.
    pub fn log_full(&self, level: Level, scope: &str, msg: &str, app_name: Option<&str>) {
        if level < self.min_level {
            return;
        }

        let record = LogRecord {
            level,
            scope: scope.to_string(),
            message: msg.to_string(),
            values: FormatValues::new(),
            label_override: None,
            app_name: app_name
                .map(ToString::to_string)
                .or_else(|| self.app_name.clone()),
            raw: false,
        };

        for output in &self.outputs {
            let _ = output.write(&record);
        }
    }

    /// Logs a trace message.
    pub fn trace(&self, scope: &str, msg: &str) {
        self.log(Level::Trace, scope, msg);
    }

    /// Logs a debug message.
    pub fn debug(&self, scope: &str, msg: &str) {
        self.log(Level::Debug, scope, msg);
    }

    /// Logs an info message.
    pub fn info(&self, scope: &str, msg: &str) {
        self.log(Level::Info, scope, msg);
    }

    /// Logs a warning message.
    pub fn warn(&self, scope: &str, msg: &str) {
        self.log(Level::Warn, scope, msg);
    }

    /// Logs an error message.
    pub fn error(&self, scope: &str, msg: &str) {
        self.log(Level::Error, scope, msg);
    }

    /// Prints a message that bypasses level filtering.
    ///
    /// Use this for command output (stats, themes, etc.) that should always be
    /// visible regardless of the configured log level. Formats like INFO but
    /// ignores `min_level`.
    pub fn print(&self, scope: &str, msg: &str) {
        let record = LogRecord {
            level: Level::Info,
            scope: scope.to_string(),
            message: msg.to_string(),
            values: FormatValues::new(),
            label_override: None,
            app_name: self.app_name.clone(),
            raw: false,
        };

        for output in &self.outputs {
            let _ = output.write(&record);
        }
    }

    /// Outputs raw text without log formatting (no tag, icon, scope).
    ///
    /// Useful for list items, continuation lines, etc. where log prefixes would be noisy.
    pub fn raw(&self, msg: &str) {
        let record = LogRecord {
            level: Level::Info,
            scope: String::new(),
            message: msg.to_string(),
            values: FormatValues::new(),
            label_override: None,
            app_name: None,
            raw: true,
        };

        for output in &self.outputs {
            let _ = output.write(&record);
        }
    }

    /// Logs a message using a preset.
    #[must_use]
    pub fn preset(&self, name: &str) -> bool {
        let Some(preset) = self.presets.get(name) else {
            internal::warn("LOGGER", &format!("Preset not found: {name}"));
            return false;
        };

        let level: Level = preset.level.parse().unwrap_or(Level::Info);
        let scope = preset.scope.as_deref().unwrap_or("LOG");

        self.log_full(level, scope, &preset.msg, preset.app_name.as_deref());
        true
    }

    /// Checks if a preset exists.
    #[must_use]
    pub fn has_preset(&self, name: &str) -> bool {
        self.presets.contains_key(name)
    }

    /// Returns the number of presets.
    #[must_use]
    pub fn preset_count(&self) -> usize {
        self.presets.len()
    }

    /// Flushes all outputs.
    ///
    /// # Errors
    /// Returns the first error encountered.
    pub fn flush(&self) -> Result<(), OutputError> {
        for output in &self.outputs {
            output.flush()?;
        }
        Ok(())
    }

    /// Returns the minimum log level.
    #[must_use]
    pub const fn min_level(&self) -> Level {
        self.min_level
    }

    /// Returns the number of outputs.
    #[must_use]
    pub fn output_count(&self) -> usize {
        self.outputs.len()
    }
}
