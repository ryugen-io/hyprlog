//! Preset loading and execution.

use hl_core::{Config, Level, Logger, internal};

/// Error type for preset operations.
#[derive(Debug)]
pub enum PresetError {
    /// Preset not found.
    NotFound(String),
    /// Invalid level in preset.
    InvalidLevel(String),
}

impl std::fmt::Display for PresetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(name) => write!(f, "preset not found: {name}"),
            Self::InvalidLevel(level) => write!(f, "invalid level in preset: {level}"),
        }
    }
}

impl std::error::Error for PresetError {}

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
    pub fn run(&self, name: &str) -> Result<(), PresetError> {
        internal::trace("PRESET", &format!("Looking up preset: {name}"));
        let preset = self.config.presets.get(name).ok_or_else(|| {
            internal::warn("PRESET", &format!("Preset not found: {name}"));
            PresetError::NotFound(name.to_string())
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
            .map_err(|_| PresetError::InvalidLevel(preset.level.clone()))?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_not_found() {
        let config = Config::default();
        let logger = Logger::builder().build();
        let runner = PresetRunner::new(&config, &logger);

        let result = runner.run("nonexistent");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PresetError::NotFound(_)));
    }

    #[test]
    fn list_empty_presets() {
        let config = Config::default();
        let logger = Logger::builder().build();
        let runner = PresetRunner::new(&config, &logger);

        assert!(runner.list().is_empty());
    }

    #[test]
    fn preset_exists_check() {
        let config = Config::default();
        let logger = Logger::builder().build();
        let runner = PresetRunner::new(&config, &logger);

        assert!(!runner.exists("startup"));
    }
}
