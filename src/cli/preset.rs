//! Repetitive log commands with fixed level/scope/message are tedious to type —
//! presets let users define them once in config and invoke by name.

use crate::config::Config;
use crate::internal;
use crate::level::Level;
use crate::logger::Logger;

/// Borrows config and logger to avoid cloning — presets are short-lived operations, not long-running state.
pub struct PresetRunner<'a> {
    config: &'a Config,
    logger: &'a Logger,
}

impl<'a> PresetRunner<'a> {
    /// Both config and logger come from the caller — the runner doesn't own any state.
    #[must_use]
    pub const fn new(config: &'a Config, logger: &'a Logger) -> Self {
        Self { config, logger }
    }

    /// Looks up the named preset in config, validates its level string, and emits the log entry.
    ///
    /// # Errors
    /// Fails early if the name doesn't exist or the level string can't be parsed — avoids silent misconfiguration.
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

    /// The shell and CLI both need to enumerate presets — returning name+app pairs lets callers group by app.
    #[must_use]
    pub fn list(&self) -> Vec<(&str, Option<&str>)> {
        self.config
            .presets
            .iter()
            .map(|(k, v)| (k.as_str(), v.app_name.as_deref()))
            .collect()
    }

    /// Pre-check avoids running a preset that will immediately fail with "not found".
    #[must_use]
    pub fn exists(&self, name: &str) -> bool {
        self.config.presets.contains_key(name)
    }
}
