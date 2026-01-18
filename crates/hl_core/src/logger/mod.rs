//! Main logger struct with builder pattern.

mod builder;

pub use builder::{FileBuilder, LoggerBuilder, TerminalBuilder};

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
}

impl Logger {
    /// Creates a new logger builder.
    #[must_use]
    pub fn builder() -> LoggerBuilder {
        LoggerBuilder::new()
    }

    /// Creates a logger from the default hyprlog config file.
    ///
    /// Loads config from `~/.config/hypr/hyprlog.conf` and builds a logger
    /// with terminal and file outputs as configured.
    ///
    /// # Arguments
    /// * `app_name` - Application name override (used for file paths/logs).
    #[must_use]
    pub fn from_config(app_name: &str) -> Self {
        internal::debug("LOGGER", "Building logger from config");
        let config = crate::config::Config::load().unwrap_or_default();
        Self::from_config_with(&config, app_name)
    }

    /// Creates a logger from a given config.
    ///
    /// # Arguments
    /// * `config` - The hyprlog config to use.
    /// * `app_name` - Application name override.
    #[must_use]
    pub fn from_config_with(config: &crate::config::Config, app_name: &str) -> Self {
        let mut builder = LoggerBuilder::new().level(config.parse_level());
        let mut output_count = 0;

        if config.terminal.enabled {
            internal::debug(
                "LOGGER",
                &format!(
                    "Terminal: colors={}, structure={}",
                    config.terminal.colors, config.terminal.structure
                ),
            );
            let mut icon_set = match config.parse_icon_type() {
                crate::fmt::IconType::NerdFont => crate::fmt::IconSet::nerdfont(),
                crate::fmt::IconType::Ascii => crate::fmt::IconSet::ascii(),
                crate::fmt::IconType::None => crate::fmt::IconSet::none(),
            };

            // Apply overrides from config
            let overrides = match config.parse_icon_type() {
                crate::fmt::IconType::NerdFont => &config.icons.nerdfont,
                crate::fmt::IconType::Ascii => &config.icons.ascii,
                crate::fmt::IconType::None => &HashMap::new(),
            };

            for (level_str, icon) in overrides {
                if let Ok(level) = level_str.parse::<Level>() {
                    icon_set.set(level, icon);
                } else {
                    internal::warn(
                        "LOGGER",
                        &format!("Invalid level in icon config: {level_str}"),
                    );
                }
            }
            let tag_config = crate::fmt::TagConfig::new()
                .prefix(&config.tag.prefix)
                .suffix(&config.tag.suffix)
                .transform(config.parse_transform())
                .min_width(config.tag.min_width)
                .alignment(config.parse_alignment());

            builder = builder
                .terminal()
                .colors(config.terminal.colors)
                .icons(icon_set)
                .structure(&config.terminal.structure)
                .tag_config(tag_config)
                .done();
            output_count += 1;
        }

        if config.file.enabled {
            internal::debug(
                "LOGGER",
                &format!(
                    "File: base_dir={}, app={}",
                    config.file.base_dir,
                    config.general.app_name.as_deref().unwrap_or(app_name)
                ),
            );
            builder = builder
                .file()
                .base_dir(&config.file.base_dir)
                .path_structure(&config.file.path_structure)
                .filename_structure(&config.file.filename_structure)
                .content_structure(&config.file.content_structure)
                .timestamp_format(&config.file.timestamp_format)
                .timestamp_format(&config.file.timestamp_format)
                .app_name(config.general.app_name.as_deref().unwrap_or(app_name))
                .done();
            output_count += 1;
        }

        internal::debug(
            "LOGGER",
            &format!("Logger built with {output_count} outputs"),
        );

        if !config.presets.is_empty() {
            internal::debug(
                "LOGGER",
                &format!("Loading {} presets", config.presets.len()),
            );
        }

        builder.presets(config.presets.clone()).build()
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
            app_name: None,
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
            app_name: None,
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
            app_name: app_name.map(ToString::to_string),
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
