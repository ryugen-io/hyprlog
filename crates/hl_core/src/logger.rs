//! Main logger struct with builder pattern.

use crate::Level;
use crate::format::FormatValues;
use crate::output::{FileOutput, LogRecord, Output, OutputError, TerminalOutput};

/// The main logger.
#[derive(Default)]
pub struct Logger {
    min_level: Level,
    outputs: Vec<Box<dyn Output>>,
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
        };

        for output in &self.outputs {
            // Ignore output errors for now (logging shouldn't panic)
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

/// Builder for configuring a logger.
#[derive(Default)]
pub struct LoggerBuilder {
    min_level: Level,
    outputs: Vec<Box<dyn Output>>,
}

impl LoggerBuilder {
    /// Creates a new logger builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            min_level: Level::Info,
            outputs: Vec::new(),
        }
    }

    /// Sets the minimum log level.
    #[must_use]
    pub const fn level(mut self, level: Level) -> Self {
        self.min_level = level;
        self
    }

    /// Adds a terminal output with default configuration.
    #[must_use]
    pub fn terminal(self) -> TerminalBuilder {
        TerminalBuilder {
            parent: self,
            output: TerminalOutput::new(),
        }
    }

    /// Adds a file output with default configuration.
    #[must_use]
    pub fn file(self) -> FileBuilder {
        FileBuilder {
            parent: self,
            output: FileOutput::new(),
        }
    }

    /// Adds a custom output.
    #[must_use]
    pub fn output(mut self, output: impl Output + 'static) -> Self {
        self.outputs.push(Box::new(output));
        self
    }

    /// Builds the logger.
    #[must_use]
    pub fn build(self) -> Logger {
        Logger {
            min_level: self.min_level,
            outputs: self.outputs,
        }
    }
}

/// Builder for terminal output configuration.
pub struct TerminalBuilder {
    parent: LoggerBuilder,
    output: TerminalOutput,
}

impl TerminalBuilder {
    /// Enables or disables colors.
    #[must_use]
    pub fn colors(mut self, enabled: bool) -> Self {
        self.output = self.output.colors(enabled);
        self
    }

    /// Sets the icon set.
    #[must_use]
    pub fn icons(mut self, icons: crate::icon::IconSet) -> Self {
        self.output = self.output.icons(icons);
        self
    }

    /// Sets the output template.
    #[must_use]
    pub fn structure(mut self, template: &str) -> Self {
        self.output = self.output.template(template);
        self
    }

    /// Sets the tag configuration.
    #[must_use]
    pub fn tag_config(mut self, config: crate::tag::TagConfig) -> Self {
        self.output = self.output.tag_config(config);
        self
    }

    /// Finishes terminal configuration and returns to the logger builder.
    #[must_use]
    pub fn done(mut self) -> LoggerBuilder {
        self.parent.outputs.push(Box::new(self.output));
        self.parent
    }
}

/// Builder for file output configuration.
pub struct FileBuilder {
    parent: LoggerBuilder,
    output: FileOutput,
}

impl FileBuilder {
    /// Sets the base directory.
    #[must_use]
    pub fn base_dir(mut self, dir: impl Into<String>) -> Self {
        self.output = self.output.base_dir(dir);
        self
    }

    /// Sets the path structure template.
    #[must_use]
    pub fn path_structure(mut self, template: &str) -> Self {
        self.output = self.output.path_structure(template);
        self
    }

    /// Sets the filename structure template.
    #[must_use]
    pub fn filename_structure(mut self, template: &str) -> Self {
        self.output = self.output.filename_structure(template);
        self
    }

    /// Sets the content structure template.
    #[must_use]
    pub fn content_structure(mut self, template: &str) -> Self {
        self.output = self.output.content_structure(template);
        self
    }

    /// Sets the application name.
    #[must_use]
    pub fn app_name(mut self, name: impl Into<String>) -> Self {
        self.output = self.output.app_name(name);
        self
    }

    /// Sets the timestamp format.
    #[must_use]
    pub fn timestamp_format(mut self, format: impl Into<String>) -> Self {
        self.output = self.output.timestamp_format(format);
        self
    }

    /// Finishes file configuration and returns to the logger builder.
    #[must_use]
    pub fn done(mut self) -> LoggerBuilder {
        self.parent.outputs.push(Box::new(self.output));
        self.parent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_default() {
        let logger = Logger::builder().build();
        assert_eq!(logger.min_level(), Level::Info);
        assert_eq!(logger.output_count(), 0);
    }

    #[test]
    fn builder_with_level() {
        let logger = Logger::builder().level(Level::Debug).build();
        assert_eq!(logger.min_level(), Level::Debug);
    }

    #[test]
    fn builder_with_terminal() {
        let logger = Logger::builder().terminal().colors(false).done().build();
        assert_eq!(logger.output_count(), 1);
    }

    #[test]
    fn builder_with_file() {
        let logger = Logger::builder()
            .file()
            .base_dir("/tmp/test")
            .app_name("test")
            .done()
            .build();
        assert_eq!(logger.output_count(), 1);
    }

    #[test]
    fn builder_multiple_outputs() {
        let logger = Logger::builder()
            .level(Level::Trace)
            .terminal()
            .done()
            .file()
            .done()
            .build();
        assert_eq!(logger.output_count(), 2);
    }

    #[test]
    fn log_respects_level() {
        let logger = Logger::builder().level(Level::Warn).build();
        // This should not panic even without outputs
        logger.info("TEST", "should be filtered");
        logger.warn("TEST", "should pass");
    }
}
