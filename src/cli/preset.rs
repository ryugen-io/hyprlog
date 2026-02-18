//! Preset loading and execution.

use crate::config::Config;
use crate::internal;
use crate::level::Level;
use crate::logger::Logger;

/// Runs presets from configuration.
pub struct PresetRunner<'a> {
    config: &'a Config,
    logger: &'a Logger,
}

impl<'a> PresetRunner<'a> {
    /// Creates a new preset runner.
    #[must_use]
    pub const fn new(config: &'a Config, logger: &'a Logger) -> Self {
        Self { config, logger }
    }

    /// Runs a preset by name.
    ///
    /// # Errors
    /// Returns error if preset not found or has invalid level.
    pub fn run(&self, name: &str) -> Result<(), crate::Error> {
        internal::trace("PRESET", &format!("Looking up preset: {name}"));
        let preset = self.config.presets.get(name).ok_or_else(|| {
            internal::warn("PRESET", &format!("Preset not found: {name}"));
            crate::Error::PresetNotFound(name.to_string())
        })?;

        internal::debug(
            "PRESET",
            &format!(
                "Preset: level={}, scope={}",
                preset.level,
                preset.scope.as_deref().unwrap_or("LOG")
            ),
        );

        let level: Level = preset
            .level
            .parse()
            .map_err(|_| crate::Error::InvalidLevel(preset.level.clone()))?;

        let scope = preset.scope.as_deref().unwrap_or("LOG");
        let app_name = preset.app_name.as_deref();

        self.logger.log_full(level, scope, &preset.msg, app_name);
        internal::info("PRESET", &format!("Executed preset: {name}"));

        Ok(())
    }

    /// Lists available presets with optional app name.
    #[must_use]
    pub fn list(&self) -> Vec<(&str, Option<&str>)> {
        self.config
            .presets
            .iter()
            .map(|(k, v)| (k.as_str(), v.app_name.as_deref()))
            .collect()
    }

    /// Checks if a preset exists.
    #[must_use]
    pub fn exists(&self, name: &str) -> bool {
        self.config.presets.contains_key(name)
    }
}
