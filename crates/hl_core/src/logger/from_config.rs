//! Logger configuration from hyprlog config files.

use super::{Logger, LoggerBuilder};
use crate::internal;
use crate::level::Level;

impl Logger {
    /// Creates a logger from the default hyprlog config file.
    ///
    /// Loads config from `~/.config/hypr/hyprlog.conf` and builds a logger
    /// with terminal and file outputs as configured.
    ///
    /// # Arguments
    /// * `app_name` - Application name override (used for file paths/logs).
    #[must_use]
    pub fn from_config(app_name: &str) -> Self {
        internal::debug("LOGGER", "Building logger from config");
        let config = crate::config::Config::load().unwrap_or_default();
        Self::from_config_with(&config, app_name)
    }

    /// Creates a logger from a given config.
    ///
    /// # Arguments
    /// * `config` - The hyprlog config to use.
    /// * `app_name` - Application name override.
    ///
    /// This method applies app-specific overrides from `[apps.{app_name}]` sections.
    #[must_use]
    pub fn from_config_with(config: &crate::config::Config, app_name: &str) -> Self {
        internal::debug("LOGGER", &format!("Initializing logger for app={app_name}"));

        // Apply app-specific config overrides
        let config = config.for_app(app_name);
        internal::debug("LOGGER", &format!("Log level: {}", config.general.level));

        let mut builder = LoggerBuilder::new().level(config.parse_level());
        let mut outputs: Vec<&str> = Vec::new();

        if config.terminal.enabled {
            builder = Self::configure_terminal(builder, &config);
            outputs.push("terminal");
        }

        if config.file.enabled {
            builder = Self::configure_file(builder, &config, app_name);
            outputs.push("file");
        }

        if config.json.enabled {
            builder = Self::configure_json(builder, &config, app_name);
            outputs.push("json");
        }

        if outputs.is_empty() {
            internal::warn("LOGGER", "No outputs enabled");
        } else {
            internal::debug(
                "LOGGER",
                &format!("Outputs enabled: [{}]", outputs.join(", ")),
            );
        }

        if !config.presets.is_empty() {
            internal::debug(
                "PRESETS",
                &format!("Loaded {} presets", config.presets.len()),
            );
        }

        internal::debug("LOGGER", "Logger ready");
        let mut logger = builder.presets(config.presets).build();
        logger.app_name = Some(app_name.to_string());
        logger
    }

    /// Configures terminal output from config.
    fn configure_terminal(builder: LoggerBuilder, config: &crate::config::Config) -> LoggerBuilder {
        internal::debug("TERMINAL", "Configuring terminal output...");
        internal::debug(
            "TERMINAL",
            &format!(
                "Colors: {}",
                if config.terminal.colors {
                    "enabled"
                } else {
                    "disabled"
                }
            ),
        );
        internal::debug("TERMINAL", &format!("Icons: {}", config.terminal.icons));

        if config.highlight.enabled {
            let patterns: Vec<&str> = [
                config.highlight.patterns.urls.as_ref().map(|_| "urls"),
                config.highlight.patterns.paths.as_ref().map(|_| "paths"),
                config.highlight.patterns.quoted.as_ref().map(|_| "quoted"),
                config
                    .highlight
                    .patterns
                    .numbers
                    .as_ref()
                    .map(|_| "numbers"),
            ]
            .into_iter()
            .flatten()
            .collect();

            internal::debug(
                "HIGHLIGHT",
                &format!("Keywords: {} loaded", config.highlight.keywords.len()),
            );
            internal::debug("HIGHLIGHT", &format!("Patterns: [{}]", patterns.join(", ")));
        } else {
            internal::debug("HIGHLIGHT", "Disabled");
        }

        let icon_set = Self::build_icon_set(config);
        let tag_config = Self::build_tag_config(config);
        let scope_config = Self::build_scope_config(config);
        let message_transform = config.parse_message_transform();

        let mut terminal = builder
            .terminal()
            .colors(config.terminal.colors)
            .icons(icon_set)
            .structure(&config.terminal.structure)
            .tag_config(tag_config)
            .scope_config(scope_config)
            .message_transform(message_transform)
            .highlight_config(config.highlight.clone());

        // Apply custom colors from config
        for name in config.colors.keys() {
            if let Some(color) = config.get_color(name) {
                terminal = terminal.color(name, color);
            }
        }

        // Apply level colors from config (e.g., colors.info = "#50fa7b")
        for level in [
            Level::Trace,
            Level::Debug,
            Level::Info,
            Level::Warn,
            Level::Error,
        ] {
            let level_name = level.as_str().to_lowercase();
            if let Some(color) = config.get_color(&level_name) {
                terminal = terminal.level_color(level, color);
            }
        }

        terminal.done()
    }

    /// Builds icon set from config.
    fn build_icon_set(config: &crate::config::Config) -> crate::fmt::IconSet {
        let mut icon_set = match config.parse_icon_type() {
            crate::fmt::IconType::NerdFont => crate::fmt::IconSet::nerdfont(),
            crate::fmt::IconType::Ascii => crate::fmt::IconSet::ascii(),
            crate::fmt::IconType::None => crate::fmt::IconSet::none(),
        };

        let overrides = match config.parse_icon_type() {
            crate::fmt::IconType::NerdFont => &config.icons.nerdfont,
            crate::fmt::IconType::Ascii => &config.icons.ascii,
            crate::fmt::IconType::None => return icon_set,
        };

        for (level_str, icon) in overrides {
            if let Ok(level) = level_str.parse::<Level>() {
                icon_set.set(level, icon);
            } else {
                internal::warn(
                    "LOGGER",
                    &format!("Invalid level in icon config: {level_str}"),
                );
            }
        }

        icon_set
    }

    /// Builds tag config from config.
    fn build_tag_config(config: &crate::config::Config) -> crate::fmt::TagConfig {
        let mut tag_config = crate::fmt::TagConfig::new()
            .prefix(&config.tag.prefix)
            .suffix(&config.tag.suffix)
            .transform(config.parse_transform())
            .min_width(config.tag.min_width)
            .alignment(config.parse_alignment());

        for (level_str, label) in &config.tag.labels {
            if let Ok(level) = level_str.parse::<Level>() {
                tag_config = tag_config.label(level, label);
            } else {
                internal::warn(
                    "LOGGER",
                    &format!("Invalid level in tag.labels: {level_str}"),
                );
            }
        }

        tag_config
    }

    /// Builds scope config from config.
    fn build_scope_config(config: &crate::config::Config) -> crate::fmt::ScopeConfig {
        crate::fmt::ScopeConfig::new()
            .min_width(config.scope.min_width)
            .alignment(config.parse_scope_alignment())
            .transform(config.parse_scope_transform())
    }

    /// Configures file output from config.
    fn configure_file(
        builder: LoggerBuilder,
        config: &crate::config::Config,
        app_name: &str,
    ) -> LoggerBuilder {
        internal::debug("FILE", "Configuring file output...");
        internal::debug("FILE", &format!("Base dir: {}", config.file.base_dir));
        internal::debug(
            "FILE",
            &format!(
                "App name: {}",
                config.general.app_name.as_deref().unwrap_or(app_name)
            ),
        );

        builder
            .file()
            .base_dir(&config.file.base_dir)
            .path_structure(&config.file.path_structure)
            .filename_structure(&config.file.filename_structure)
            .content_structure(&config.file.content_structure)
            .timestamp_format(&config.file.timestamp_format)
            .app_name(config.general.app_name.as_deref().unwrap_or(app_name))
            .done()
    }

    /// Configures JSON database output from config.
    fn configure_json(
        builder: LoggerBuilder,
        config: &crate::config::Config,
        app_name: &str,
    ) -> LoggerBuilder {
        internal::debug("JSON", "Configuring JSON database output...");
        internal::debug("JSON", &format!("Path: {}", config.json.path));

        builder
            .json()
            .path(&config.json.path)
            .app_name(config.general.app_name.as_deref().unwrap_or(app_name))
            .done()
    }
}
