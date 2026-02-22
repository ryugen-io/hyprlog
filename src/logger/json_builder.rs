//! JSON output has its own concerns (JSONL path, app name) that don't belong on the main `LoggerBuilder`.

use super::LoggerBuilder;
use crate::output::JsonOutput;
use std::path::PathBuf;

/// Sub-builder pattern keeps JSON-specific options off the main `LoggerBuilder`.
pub struct JsonBuilder {
    pub(super) parent: LoggerBuilder,
    pub(super) output: JsonOutput,
}

impl JsonBuilder {
    /// Default XDG path doesn't work for every deployment (containers, custom setups).
    #[must_use]
    pub fn path(mut self, path: impl Into<PathBuf>) -> Self {
        self.output = self.output.path(path);
        self
    }

    /// JSONL records need an app field so stats queries can filter by application.
    #[must_use]
    pub fn app_name(mut self, name: impl Into<String>) -> Self {
        self.output = self.output.app_name(name);
        self
    }

    /// Sub-builder consumes self — returning the parent lets the user chain more outputs.
    #[must_use]
    pub fn done(mut self) -> LoggerBuilder {
        self.parent.outputs.push(Box::new(self.output));
        self.parent
    }
}
