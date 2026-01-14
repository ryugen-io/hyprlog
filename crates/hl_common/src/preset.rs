//! Preset loading and execution.

use hl_core::{Config, Level, Logger};

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
        let preset = self
            .config
            .presets
            .get(name)
            .ok_or_else(|| PresetError::NotFound(name.to_string()))?;

        let level: Level = preset
            .level
            .parse()
            .map_err(|_| PresetError::InvalidLevel(preset.level.clone()))?;

        let scope = preset.scope.as_deref().unwrap_or("LOG");

        self.logger.log(level, scope, &preset.msg);

        Ok(())
    }

    /// Lists available preset names.
    #[must_use]
    pub fn list(&self) -> Vec<&str> {
        self.config.presets.keys().map(String::as_str).collect()
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
