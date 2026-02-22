//! Direct construction would require knowing every output's internals — the builder
//! hides that behind a stepwise API, and the resulting Logger fans out each record
//! to all configured outputs.

mod builder;
mod from_config;
mod json_builder;

pub use builder::{FileBuilder, LoggerBuilder, TerminalBuilder};
pub use json_builder::JsonBuilder;

use crate::config::PresetConfig;
use crate::fmt::FormatValues;
use crate::internal;
use crate::level::Level;
use crate::output::{LogRecord, Output};
use std::collections::HashMap;

/// Immutable after build — guarantees thread-safe concurrent logging without locks.
#[derive(Default)]
pub struct Logger {
    min_level: Level,
    outputs: Vec<Box<dyn Output>>,
    presets: HashMap<String, PresetConfig>,
    pub(crate) app_name: Option<String>,
}

impl Logger {
    /// Direct construction would expose output internals — the builder provides a guided API instead.
    #[must_use]
    pub fn builder() -> LoggerBuilder {
        LoggerBuilder::new()
    }

    /// Core dispatch — filters by severity, then fans out to all configured outputs.
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

    /// Presets and custom events use domain-specific labels ("SUCCESS", "DEPLOY") instead of built-in level names.
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

    /// Presets can target a different app's log directory — the app name must be overridable per call.
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

    /// High-volume instrumentation that should vanish in production builds.
    pub fn trace(&self, scope: &str, msg: &str) {
        self.log(Level::Trace, scope, msg);
    }

    /// Development-time diagnostics that are too noisy for normal operation.
    pub fn debug(&self, scope: &str, msg: &str) {
        self.log(Level::Debug, scope, msg);
    }

    /// Normal operational milestones — config loaded, listener started, etc.
    pub fn info(&self, scope: &str, msg: &str) {
        self.log(Level::Info, scope, msg);
    }

    /// Non-fatal anomalies — missing optional config, deprecated features, recoverable errors.
    pub fn warn(&self, scope: &str, msg: &str) {
        self.log(Level::Warn, scope, msg);
    }

    /// Unrecoverable failures — I/O errors, invalid state, broken invariants.
    pub fn error(&self, scope: &str, msg: &str) {
        self.log(Level::Error, scope, msg);
    }

    /// Command output (stats, themes, cleanup results) must always be visible —
    /// level filtering would hide the results the user explicitly asked for.
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

    /// List items and continuation lines would look broken with repeated `[INFO] SCOPE` prefixes.
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

    /// Presets avoid retyping level, scope, and text for repetitive log messages (startup, shutdown, deploy).
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

    /// Callers may want to validate a preset name before attempting to use it.
    #[must_use]
    pub fn has_preset(&self, name: &str) -> bool {
        self.presets.contains_key(name)
    }

    /// CLI help and diagnostics need to report how many presets are loaded.
    #[must_use]
    pub fn preset_count(&self) -> usize {
        self.presets.len()
    }

    /// Buffered outputs (file, JSON) may lose tail data on abrupt exit without an explicit flush.
    ///
    /// # Errors
    /// Returns the first I/O error encountered across all outputs.
    pub fn flush(&self) -> Result<(), crate::Error> {
        for output in &self.outputs {
            output.flush()?;
        }
        Ok(())
    }

    /// Tests and diagnostics need to verify which severity threshold is active.
    #[must_use]
    pub const fn min_level(&self) -> Level {
        self.min_level
    }

    /// Tests verify the builder wired up the expected number of backends.
    #[must_use]
    pub fn output_count(&self) -> usize {
        self.outputs.len()
    }
}
