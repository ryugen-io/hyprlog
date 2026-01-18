//! Optional TOML configuration for hyprlog.

mod error;
mod structs;

pub use error::ConfigError;
pub use structs::{
    CleanupConfig, FileConfig, GeneralConfig, IconsConfig, PresetConfig, RetentionConfig,
    ShellConfig, TagConfigFile, TerminalConfig,
};

use crate::fmt::{Alignment, Color, IconType, Transform};
use crate::internal;
use crate::level::Level;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Root configuration structure.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    /// General settings.
    pub general: GeneralConfig,
    /// Terminal output settings.
    pub terminal: TerminalConfig,
    /// Shell settings.
    pub shell: ShellConfig,
    /// File output settings.
    pub file: FileConfig,
    /// Cleanup settings.
    pub cleanup: CleanupConfig,
    /// Tag formatting settings.
    pub tag: TagConfigFile,
    /// Color definitions.
    pub colors: HashMap<String, String>,
    /// Icon definitions per level.
    pub icons: IconsConfig,
    /// Log presets/dictionary.
    pub presets: HashMap<String, PresetConfig>,
}

/// Extracts `source = "path"` lines from config content.
fn extract_sources(content: &str) -> (Vec<String>, String) {
    let mut sources = Vec::new();
    let mut remaining = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("source") && trimmed.contains('=') {
            if let Some(path) = trimmed
                .split('=')
                .nth(1)
                .map(|s| s.trim().trim_matches('"').trim_matches('\''))
            {
                if !path.is_empty() {
                    sources.push(path.to_string());
                }
            }
        } else {
            remaining.push_str(line);
            remaining.push('\n');
        }
    }

    (sources, remaining)
}

impl Config {
    /// Loads configuration from the default location.
    ///
    /// Looks for `~/.config/hypr/hyprlog.conf`.
    /// Supports Hyprland-style `source = "path"` directives.
    ///
    /// # Errors
    /// Returns error if config cannot be loaded.
    pub fn load() -> Result<Self, ConfigError> {
        internal::debug("CONFIG", "Loading config from default location");
        let config_path = Self::get_config_path()?;
        let config = Self::load_with_sources(&config_path, &mut HashSet::new())?;
        internal::info(
            "CONFIG",
            &format!("Config loaded from {}", config_path.display()),
        );
        Ok(config)
    }

    /// Loads configuration from a specific path.
    ///
    /// # Errors
    /// Returns error if file cannot be read or parsed.
    pub fn load_from(path: &Path) -> Result<Self, ConfigError> {
        Self::load_with_sources(path, &mut HashSet::new())
    }

    /// Loads configuration with source file processing and cycle detection.
    fn load_with_sources(path: &Path, seen: &mut HashSet<PathBuf>) -> Result<Self, ConfigError> {
        if !path.exists() {
            internal::debug("CONFIG", "Config file not found, using defaults");
            return Ok(Self::default());
        }

        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        if !seen.insert(canonical.clone()) {
            internal::warn(
                "CONFIG",
                &format!("Cyclic include detected: {}", canonical.display()),
            );
            return Err(ConfigError::CyclicInclude(canonical));
        }

        let content = fs::read_to_string(path)?;
        let (sources, toml_content) = extract_sources(&content);
        let mut config: Self = toml::from_str(&toml_content)?;

        for source_path in sources {
            internal::debug("CONFIG", &format!("Processing source: {source_path}"));
            let expanded = shellexpand::tilde(&source_path);
            let source_file = Path::new(expanded.as_ref());
            if source_file.exists() {
                let source_config = Self::load_with_sources(source_file, seen)?;
                config.merge(source_config);
            } else {
                internal::warn("CONFIG", &format!("Source file not found: {source_path}"));
            }
        }

        Ok(config)
    }

    /// Merges another config into self.
    pub fn merge(&mut self, other: Self) {
        for (k, v) in other.colors {
            self.colors.entry(k).or_insert(v);
        }
        for (k, v) in other.presets {
            self.presets.entry(k).or_insert(v);
        }
        for (k, v) in other.icons.nerdfont {
            self.icons.nerdfont.entry(k).or_insert(v);
        }
        for (k, v) in other.icons.ascii {
            self.icons.ascii.entry(k).or_insert(v);
        }
        for (k, v) in other.tag.labels {
            self.tag.labels.entry(k).or_insert(v);
        }
    }

    /// Returns the default config file path.
    ///
    /// # Errors
    /// Returns error if path cannot be determined.
    pub fn get_config_path() -> Result<PathBuf, ConfigError> {
        directories::BaseDirs::new()
            .map(|dirs| dirs.config_dir().join("hypr").join("hyprlog.conf"))
            .ok_or(ConfigError::ConfigDirNotFound)
    }

    /// Parses the general level string to a Level enum.
    #[must_use]
    pub fn parse_level(&self) -> Level {
        self.general.level.parse().unwrap_or(Level::Info)
    }

    /// Parses the terminal icon type.
    #[must_use]
    pub fn parse_icon_type(&self) -> IconType {
        match self.terminal.icons.to_lowercase().as_str() {
            "ascii" => IconType::Ascii,
            "none" => IconType::None,
            _ => IconType::NerdFont,
        }
    }

    /// Parses the tag transform.
    #[must_use]
    pub fn parse_transform(&self) -> Transform {
        match self.tag.transform.to_lowercase().as_str() {
            "uppercase" | "upper" => Transform::Uppercase,
            "lowercase" | "lower" => Transform::Lowercase,
            "capitalize" | "cap" => Transform::Capitalize,
            _ => Transform::None,
        }
    }

    /// Parses the tag alignment.
    #[must_use]
    pub fn parse_alignment(&self) -> Alignment {
        match self.tag.alignment.to_lowercase().as_str() {
            "left" => Alignment::Left,
            "right" => Alignment::Right,
            _ => Alignment::Center,
        }
    }

    /// Parses a color from the colors map.
    #[must_use]
    pub fn get_color(&self, name: &str) -> Option<Color> {
        self.colors.get(name).map(|hex| Color::from_hex(hex))
    }
}
