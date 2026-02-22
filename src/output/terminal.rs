//! Terminal is the most common output — users expect immediate colored feedback on stderr/stdout
//! without configuring file paths or databases.

use crate::config::HighlightConfig;
use crate::fmt::{
    Color, FormatTemplate, FormatValues, IconSet, ScopeConfig, TagConfig, Transform, highlight,
    style,
};
use crate::level::Level;

use super::{LogRecord, Output};
use std::collections::HashMap;
use std::io::{self, Write};

/// All terminal-specific rendering state in one struct — avoids scattering colors, icons, and templates across the crate.
#[derive(Debug, Clone)]
pub struct TerminalOutput {
    /// Piped output and CI environments can't render ANSI escape codes.
    colors_enabled: bool,
    /// Not every terminal has `NerdFont` — the active icon family determines which glyphs to render.
    icons: IconSet,
    /// Level indicators need project-specific delimiters, casing, and width.
    tag_config: TagConfig,
    /// Scope names vary in length — padding and alignment keep the message column stable.
    scope_config: ScopeConfig,
    /// Some alert systems need uppercase messages for visibility.
    message_transform: Transform,
    /// Different use cases need different column layouts per log line.
    template: FormatTemplate,
    /// Named colors decouple themes from hardcoded hex values — `<red>` resolves through this map.
    color_map: HashMap<String, Color>,
    /// Default level colors may clash with the user's terminal theme — overrides fix that.
    level_colors: HashMap<Level, Color>,
    /// Dense log output buries URLs, paths, and numbers without auto-highlighting.
    highlight_config: HighlightConfig,
}

impl Default for TerminalOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalOutput {
    /// Sensible defaults (colors on, `NerdFont` icons, Dracula-ish palette) work for most Hyprland setups.
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
            scope_config: ScopeConfig::default(),
            message_transform: Transform::None,
            template: FormatTemplate::parse("{tag} {scope}  {msg}"),
            color_map,
            level_colors,
            highlight_config: HighlightConfig::default(),
        }
    }

    /// Piped output and CI environments can't render ANSI escape codes.
    #[must_use]
    pub const fn colors(mut self, enabled: bool) -> Self {
        self.colors_enabled = enabled;
        self
    }

    /// Not all terminals have `NerdFont` installed — the caller chooses the right icon family.
    #[must_use]
    pub fn icons(mut self, icons: IconSet) -> Self {
        self.icons = icons;
        self
    }

    /// The default `[INFO]` style may not match the project's log convention.
    #[must_use]
    pub fn tag_config(mut self, config: TagConfig) -> Self {
        self.tag_config = config;
        self
    }

    /// Scope column width and casing depend on the project's naming conventions.
    #[must_use]
    pub const fn scope_config(mut self, config: ScopeConfig) -> Self {
        self.scope_config = config;
        self
    }

    /// Some alert systems need uppercase messages for visibility.
    #[must_use]
    pub const fn message_transform(mut self, transform: Transform) -> Self {
        self.message_transform = transform;
        self
    }

    /// Different use cases need different information density per line.
    #[must_use]
    pub fn template(mut self, template: &str) -> Self {
        self.template = FormatTemplate::parse(template);
        self
    }

    /// Named colors decouple themes from hardcoded hex values.
    #[must_use]
    pub fn color(mut self, name: impl Into<String>, color: Color) -> Self {
        self.color_map.insert(name.into(), color);
        self
    }

    /// Default level colors may clash with the user's terminal theme.
    #[must_use]
    pub fn level_color(mut self, level: Level, color: Color) -> Self {
        self.level_colors.insert(level, color);
        self
    }

    /// Highlighting has a runtime cost from regex matching — callers may want to disable or customize it.
    #[must_use]
    pub fn highlight_config(mut self, config: HighlightConfig) -> Self {
        self.highlight_config = config;
        self
    }

    /// Assembles tag, icon, scope, and message into the template — the rendering hot path for every terminal log line.
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

        // Format scope (padded and dimmed)
        let padded_scope = self.scope_config.format(&record.scope);
        let scope = if self.colors_enabled {
            format!("\x1b[2m{padded_scope}\x1b[0m")
        } else {
            padded_scope
        };

        // Apply message transform and auto-highlighting
        let transformed_msg = self.message_transform.apply(&record.message);
        let msg_with_highlights = if self.colors_enabled {
            highlight::inject_tags(&transformed_msg, &self.highlight_config)
        } else {
            transformed_msg
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
    fn write(&self, record: &LogRecord) -> Result<(), crate::Error> {
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

    fn flush(&self) -> Result<(), crate::Error> {
        io::stdout().flush()?;
        io::stderr().flush()?;
        Ok(())
    }
}
