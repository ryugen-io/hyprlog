//! Terminal output with color support.

use crate::config::HighlightConfig;
use crate::fmt::{Color, FormatTemplate, FormatValues, IconSet, TagConfig, highlight, style};
use crate::level::Level;

use super::{LogRecord, Output, OutputError};
use std::collections::HashMap;
use std::io::{self, Write};

/// Terminal output configuration.
#[derive(Debug, Clone)]
pub struct TerminalOutput {
    /// Enable colored output.
    colors_enabled: bool,
    /// Icon set for levels.
    icons: IconSet,
    /// Tag formatting config.
    tag_config: TagConfig,
    /// Output structure template.
    template: FormatTemplate,
    /// Named colors for styling.
    color_map: HashMap<String, Color>,
    /// Colors per level.
    level_colors: HashMap<Level, Color>,
    /// Auto-highlighting config.
    highlight_config: HighlightConfig,
}

impl Default for TerminalOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalOutput {
    /// Creates a new terminal output with defaults.
    #[must_use]
    pub fn new() -> Self {
        let mut level_colors = HashMap::new();
        level_colors.insert(Level::Trace, Color::purple());
        level_colors.insert(Level::Debug, Color::purple());
        level_colors.insert(Level::Info, Color::cyan());
        level_colors.insert(Level::Warn, Color::yellow());
        level_colors.insert(Level::Error, Color::red());

        let mut color_map = HashMap::new();
        color_map.insert("red".to_string(), Color::red());
        color_map.insert("green".to_string(), Color::green());
        color_map.insert("yellow".to_string(), Color::yellow());
        color_map.insert("cyan".to_string(), Color::cyan());
        color_map.insert("blue".to_string(), Color::blue());
        color_map.insert("purple".to_string(), Color::purple());
        color_map.insert("pink".to_string(), Color::pink());
        color_map.insert("orange".to_string(), Color::orange());
        color_map.insert("white".to_string(), Color::white());

        Self {
            colors_enabled: true,
            icons: IconSet::nerdfont(),
            tag_config: TagConfig::default(),
            template: FormatTemplate::parse("{tag} {scope}  {msg}"),
            color_map,
            level_colors,
            highlight_config: HighlightConfig::default(),
        }
    }

    /// Enables or disables colors.
    #[must_use]
    pub const fn colors(mut self, enabled: bool) -> Self {
        self.colors_enabled = enabled;
        self
    }

    /// Sets the icon set.
    #[must_use]
    pub fn icons(mut self, icons: IconSet) -> Self {
        self.icons = icons;
        self
    }

    /// Sets the tag configuration.
    #[must_use]
    pub fn tag_config(mut self, config: TagConfig) -> Self {
        self.tag_config = config;
        self
    }

    /// Sets the output template.
    #[must_use]
    pub fn template(mut self, template: &str) -> Self {
        self.template = FormatTemplate::parse(template);
        self
    }

    /// Sets a named color.
    #[must_use]
    pub fn color(mut self, name: impl Into<String>, color: Color) -> Self {
        self.color_map.insert(name.into(), color);
        self
    }

    /// Sets the color for a level.
    #[must_use]
    pub fn level_color(mut self, level: Level, color: Color) -> Self {
        self.level_colors.insert(level, color);
        self
    }

    /// Sets the highlight configuration.
    #[must_use]
    pub fn highlight_config(mut self, config: HighlightConfig) -> Self {
        self.highlight_config = config;
        self
    }

    /// Formats and prints a log record.
    fn format_record(&self, record: &LogRecord) -> String {
        let level_color = self
            .level_colors
            .get(&record.level)
            .copied()
            .unwrap_or(Color::white());

        // Format tag with color (uses label_override if set)
        let tag = record.format_tag(&self.tag_config);
        let tag = if self.colors_enabled {
            format!("{}{}{}", level_color.fg_ansi(), tag, Color::RESET)
        } else {
            tag
        };

        // Format icon with color
        let icon = self.icons.get(record.level);
        let icon = if self.colors_enabled && !icon.is_empty() {
            format!("{}{}{}", level_color.fg_ansi(), icon, Color::RESET)
        } else {
            icon.to_string()
        };

        // Format scope (dimmed)
        let scope = if self.colors_enabled {
            format!("\x1b[2m{}\x1b[0m", record.scope)
        } else {
            record.scope.clone()
        };

        // Format message with auto-highlighting and inline styles
        let msg_with_highlights = if self.colors_enabled {
            highlight::inject_tags(&record.message, &self.highlight_config)
        } else {
            record.message.clone()
        };
        let msg_segments = style::parse(&msg_with_highlights);
        let msg = if self.colors_enabled {
            style::render(&msg_segments, &self.color_map)
        } else {
            style::render_plain(&msg_segments)
        };

        // Build values and render template
        let values = FormatValues::new()
            .tag(&tag)
            .icon(&icon)
            .scope(&scope)
            .msg(&msg)
            .level(record.level.as_str())
            .app(record.app_name.as_deref().unwrap_or("hyprlog"));

        self.template.render(&values)
    }
}

impl Output for TerminalOutput {
    fn write(&self, record: &LogRecord) -> Result<(), OutputError> {
        // Raw mode: just output the message without formatting
        if record.raw {
            writeln!(io::stdout(), "{}", record.message)?;
            return Ok(());
        }

        let formatted = self.format_record(record);

        // Warn and Error go to stderr, others to stdout
        if record.level >= Level::Warn {
            writeln!(io::stderr(), "{formatted}")?;
        } else {
            writeln!(io::stdout(), "{formatted}")?;
        }

        Ok(())
    }

    fn flush(&self) -> Result<(), OutputError> {
        io::stdout().flush()?;
        io::stderr().flush()?;
        Ok(())
    }
}
