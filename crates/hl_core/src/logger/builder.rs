//! Logger builder types.

use super::Logger;
use super::json_builder::JsonBuilder;
use crate::config::{HighlightConfig, PresetConfig};
use crate::fmt::{Color, IconSet, TagConfig};
use crate::level::Level;
use crate::output::{FileOutput, JsonOutput, Output, TerminalOutput};
use std::collections::HashMap;

/// Builder for configuring a logger.
#[derive(Default)]
pub struct LoggerBuilder {
    pub(super) min_level: Level,
    pub(super) outputs: Vec<Box<dyn Output>>,
    pub(super) presets: HashMap<String, PresetConfig>,
}

impl LoggerBuilder {
    /// Creates a new logger builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            min_level: Level::Info,
            outputs: Vec::new(),
            presets: HashMap::new(),
        }
    }

    /// Sets the minimum log level.
    #[must_use]
    pub const fn level(mut self, level: Level) -> Self {
        self.min_level = level;
        self
    }

    /// Sets the presets.
    #[must_use]
    pub fn presets(mut self, presets: HashMap<String, PresetConfig>) -> Self {
        self.presets = presets;
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

    /// Adds a JSON database output with default configuration.
    #[must_use]
    pub fn json(self) -> JsonBuilder {
        JsonBuilder {
            parent: self,
            output: JsonOutput::new(),
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
            presets: self.presets,
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
    pub fn icons(mut self, icons: IconSet) -> Self {
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
    pub fn tag_config(mut self, config: TagConfig) -> Self {
        self.output = self.output.tag_config(config);
        self
    }

    /// Sets a named color.
    #[must_use]
    pub fn color(mut self, name: impl Into<String>, color: Color) -> Self {
        self.output = self.output.color(name, color);
        self
    }

    /// Sets the color for a log level.
    #[must_use]
    pub fn level_color(mut self, level: Level, color: Color) -> Self {
        self.output = self.output.level_color(level, color);
        self
    }

    /// Sets the highlight configuration.
    #[must_use]
    pub fn highlight_config(mut self, config: HighlightConfig) -> Self {
        self.output = self.output.highlight_config(config);
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
