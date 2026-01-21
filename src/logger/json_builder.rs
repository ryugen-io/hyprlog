//! JSON database output builder.

use super::LoggerBuilder;
use crate::output::JsonOutput;
use std::path::PathBuf;

/// Builder for JSON database output configuration.
pub struct JsonBuilder {
    pub(super) parent: LoggerBuilder,
    pub(super) output: JsonOutput,
}

impl JsonBuilder {
    /// Sets the output file path.
    #[must_use]
    pub fn path(mut self, path: impl Into<PathBuf>) -> Self {
        self.output = self.output.path(path);
        self
    }

    /// Sets the application name.
    #[must_use]
    pub fn app_name(mut self, name: impl Into<String>) -> Self {
        self.output = self.output.app_name(name);
        self
    }

    /// Finishes JSON configuration and returns to the logger builder.
    #[must_use]
    pub fn done(mut self) -> LoggerBuilder {
        self.parent.outputs.push(Box::new(self.output));
        self.parent
    }
}
