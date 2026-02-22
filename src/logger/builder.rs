//! Direct Logger construction would require knowing every output's internals —
//! the builder hides that behind a stepwise API.

use super::Logger;
use super::json_builder::JsonBuilder;
use crate::config::{HighlightConfig, PresetConfig};
use crate::fmt::{Color, IconSet, ScopeConfig, TagConfig, Transform};
use crate::level::Level;
use crate::output::{FileOutput, JsonOutput, Output, TerminalOutput};
use std::collections::HashMap;

/// Direct Logger construction would expose output internals to every caller.
#[derive(Default)]
pub struct LoggerBuilder {
    pub(super) min_level: Level,
    pub(super) outputs: Vec<Box<dyn Output>>,
    pub(super) presets: HashMap<String, PresetConfig>,
}

impl LoggerBuilder {
    /// Info is a safe default for production — Debug/Trace are opt-in.
    #[must_use]
    pub fn new() -> Self {
        Self {
            min_level: Level::Info,
            outputs: Vec::new(),
            presets: HashMap::new(),
        }
    }

    /// Noisy low-level messages slow down production output.
    #[must_use]
    pub const fn level(mut self, level: Level) -> Self {
        self.min_level = level;
        self
    }

    /// Users define reusable log templates in config (e.g., "startup", "shutdown").
    #[must_use]
    pub fn presets(mut self, presets: HashMap<String, PresetConfig>) -> Self {
        self.presets = presets;
        self
    }

    /// Terminal output has its own concerns (colors, icons, width) needing a dedicated sub-builder.
    #[must_use]
    pub fn terminal(self) -> TerminalBuilder {
        TerminalBuilder {
            parent: self,
            output: TerminalOutput::new(),
        }
    }

    /// File output has its own concerns (paths, timestamps, rotation) needing a dedicated sub-builder.
    #[must_use]
    pub fn file(self) -> FileBuilder {
        FileBuilder {
            parent: self,
            output: FileOutput::new(),
        }
    }

    /// JSON output has its own concerns (JSONL format, database path) needing a dedicated sub-builder.
    #[must_use]
    pub fn json(self) -> JsonBuilder {
        JsonBuilder {
            parent: self,
            output: JsonOutput::new(),
        }
    }

    /// The three built-in backends can't cover every use case.
    #[must_use]
    pub fn output(mut self, output: impl Output + 'static) -> Self {
        self.outputs.push(Box::new(output));
        self
    }

    /// Immutability after build guarantees thread-safe concurrent logging.
    #[must_use]
    pub fn build(self) -> Logger {
        Logger {
            min_level: self.min_level,
            outputs: self.outputs,
            presets: self.presets,
            app_name: None,
        }
    }
}

/// Terminal output has its own set of concerns (colors, icons, width) separate from file output.
pub struct TerminalBuilder {
    parent: LoggerBuilder,
    output: TerminalOutput,
}

impl TerminalBuilder {
    /// Piped output and color-incapable terminals break on ANSI escape codes.
    #[must_use]
    pub fn colors(mut self, enabled: bool) -> Self {
        self.output = self.output.colors(enabled);
        self
    }

    /// Not all terminals have `NerdFont` installed.
    #[must_use]
    pub fn icons(mut self, icons: IconSet) -> Self {
        self.output = self.output.icons(icons);
        self
    }

    /// Different use cases need different information density per line.
    #[must_use]
    pub fn structure(mut self, template: &str) -> Self {
        self.output = self.output.template(template);
        self
    }

    /// The default `[INFO]` style may not match the project's log convention.
    #[must_use]
    pub fn tag_config(mut self, config: TagConfig) -> Self {
        self.output = self.output.tag_config(config);
        self
    }

    /// Scope column width and casing depend on the project's naming conventions.
    #[must_use]
    pub fn scope_config(mut self, config: ScopeConfig) -> Self {
        self.output = self.output.scope_config(config);
        self
    }

    /// Some alert systems need uppercase messages for visibility.
    #[must_use]
    pub fn message_transform(mut self, transform: Transform) -> Self {
        self.output = self.output.message_transform(transform);
        self
    }

    /// Named colors decouple themes from hardcoded hex values.
    #[must_use]
    pub fn color(mut self, name: impl Into<String>, color: Color) -> Self {
        self.output = self.output.color(name, color);
        self
    }

    /// Default level colors may clash with the user's terminal theme.
    #[must_use]
    pub fn level_color(mut self, level: Level, color: Color) -> Self {
        self.output = self.output.level_color(level, color);
        self
    }

    /// Paths, URLs, and numbers are hard to spot in dense log output without color hints.
    #[must_use]
    pub fn highlight_config(mut self, config: HighlightConfig) -> Self {
        self.output = self.output.highlight_config(config);
        self
    }

    /// Sub-builder consumes self, so there must be a way back to chain more outputs.
    #[must_use]
    pub fn done(mut self) -> LoggerBuilder {
        self.parent.outputs.push(Box::new(self.output));
        self.parent
    }
}

/// File output has its own set of concerns (paths, timestamps, rotation) separate from terminal.
pub struct FileBuilder {
    parent: LoggerBuilder,
    output: FileOutput,
}

impl FileBuilder {
    /// Default `~/.local/share/hypr/hyprlog` doesn't work for every deployment.
    #[must_use]
    pub fn base_dir(mut self, dir: impl Into<String>) -> Self {
        self.output = self.output.base_dir(dir);
        self
    }

    /// Different projects organize logs differently (by app, by date, flat, etc.).
    #[must_use]
    pub fn path_structure(mut self, template: &str) -> Self {
        self.output = self.output.path_structure(template);
        self
    }

    /// Multiple apps logging to the same directory need distinct filenames.
    #[must_use]
    pub fn filename_structure(mut self, template: &str) -> Self {
        self.output = self.output.filename_structure(template);
        self
    }

    /// File output doesn't need ANSI colors but may need timestamps or different column order.
    #[must_use]
    pub fn content_structure(mut self, template: &str) -> Self {
        self.output = self.output.content_structure(template);
        self
    }

    /// Without this, all apps would dump logs into the same directory.
    #[must_use]
    pub fn app_name(mut self, name: impl Into<String>) -> Self {
        self.output = self.output.app_name(name);
        self
    }

    /// Different locales and log analysis tools expect different timestamp formats.
    #[must_use]
    pub fn timestamp_format(mut self, format: impl Into<String>) -> Self {
        self.output = self.output.timestamp_format(format);
        self
    }

    /// File logs often need simpler tags than terminal for grep-friendliness.
    #[must_use]
    pub fn tag_config(mut self, config: TagConfig) -> Self {
        self.output = self.output.tag_config(config);
        self
    }

    /// Sub-builder consumes self, so there must be a way back to chain more outputs.
    #[must_use]
    pub fn done(mut self) -> LoggerBuilder {
        self.parent.outputs.push(Box::new(self.output));
        self.parent
    }
}
