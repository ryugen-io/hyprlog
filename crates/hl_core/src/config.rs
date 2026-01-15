//! Optional TOML configuration for hyprlog.

use crate::Level;
use crate::color::Color;
use crate::icon::IconType;
use crate::tag::{Alignment, Transform};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Error type for configuration operations.
#[derive(Debug)]
pub enum ConfigError {
    /// I/O error reading config file.
    Io(std::io::Error),
    /// TOML parsing error.
    Parse(toml::de::Error),
    /// Config directory not found.
    ConfigDirNotFound,
    /// Cyclic include detected.
    CyclicInclude(PathBuf),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Parse(e) => write!(f, "parse error: {e}"),
            Self::ConfigDirNotFound => write!(f, "config directory not found"),
            Self::CyclicInclude(p) => write!(f, "cyclic include: {}", p.display()),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Parse(e) => Some(e),
            Self::ConfigDirNotFound | Self::CyclicInclude(_) => None,
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(e: toml::de::Error) -> Self {
        Self::Parse(e)
    }
}

/// Root configuration structure.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    /// General settings.
    pub general: GeneralConfig,
    /// Terminal output settings.
    pub terminal: TerminalConfig,
    /// File output settings.
    pub file: FileConfig,
    /// Tag formatting settings.
    pub tag: TagConfigFile,
    /// Color definitions.
    pub colors: HashMap<String, String>,
    /// Icon definitions per level.
    pub icons: IconsConfig,
    /// Log presets/dictionary.
    pub presets: HashMap<String, PresetConfig>,
}

/// General configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    /// Minimum log level.
    pub level: String,
    /// Application name.
    pub app_name: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            app_name: "hyprlog".to_string(),
        }
    }
}

/// Terminal output configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct TerminalConfig {
    /// Enable terminal output.
    pub enabled: bool,
    /// Enable colors.
    pub colors: bool,
    /// Icon type (nerdfont, ascii, none).
    pub icons: String,
    /// Output structure template.
    pub structure: String,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            colors: true,
            icons: "nerdfont".to_string(),
            structure: "{tag} {scope}  {msg}".to_string(),
        }
    }
}

/// File output configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct FileConfig {
    /// Enable file output.
    pub enabled: bool,
    /// Base directory for logs.
    pub base_dir: String,
    /// Path structure template.
    pub path_structure: String,
    /// Filename structure template.
    pub filename_structure: String,
    /// Content structure template.
    pub content_structure: String,
    /// Timestamp format.
    pub timestamp_format: String,
    /// Retention settings.
    pub retention: RetentionConfig,
}

impl Default for FileConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            base_dir: "~/.local/share/hyprlog/logs".to_string(),
            path_structure: "{year}/{month}/{app}".to_string(),
            filename_structure: "{scope}_{level}_{day}.log".to_string(),
            content_structure: "{timestamp} {tag} {scope}  {msg}".to_string(),
            timestamp_format: "%Y-%m-%d %H:%M:%S".to_string(),
            retention: RetentionConfig::default(),
        }
    }
}

/// Log retention configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct RetentionConfig {
    /// Maximum age in days.
    pub max_age_days: u32,
    /// Maximum total size (e.g., "500M", "1G").
    pub max_total_size: Option<String>,
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            max_age_days: 30,
            max_total_size: None,
        }
    }
}

/// Tag formatting configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct TagConfigFile {
    /// Prefix before tag.
    pub prefix: String,
    /// Suffix after tag.
    pub suffix: String,
    /// Text transform (none, uppercase, lowercase, capitalize).
    pub transform: String,
    /// Minimum width.
    pub min_width: usize,
    /// Alignment (left, right, center).
    pub alignment: String,
    /// Custom labels per level.
    pub labels: HashMap<String, String>,
}

impl Default for TagConfigFile {
    fn default() -> Self {
        Self {
            prefix: "[".to_string(),
            suffix: "]".to_string(),
            transform: "uppercase".to_string(),
            min_width: 5,
            alignment: "center".to_string(),
            labels: HashMap::new(),
        }
    }
}

/// Icons configuration.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct IconsConfig {
    /// Nerd Font icons.
    pub nerdfont: HashMap<String, String>,
    /// ASCII icons.
    pub ascii: HashMap<String, String>,
}

/// Preset/dictionary entry.
#[derive(Debug, Clone, Deserialize)]
pub struct PresetConfig {
    /// Log level.
    pub level: String,
    /// Scope.
    pub scope: Option<String>,
    /// Message.
    pub msg: String,
}

/// Extracts `source = "path"` lines from config content.
///
/// Returns (sources, remaining TOML content).
/// This allows Hyprland-style multiple source lines which aren't valid TOML.
fn extract_sources(content: &str) -> (Vec<String>, String) {
    let mut sources = Vec::new();
    let mut remaining = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("source") && trimmed.contains('=') {
            // Extract path from: source = "path" or source="path"
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
        let config_path = Self::get_config_path()?;
        Self::load_with_sources(&config_path, &mut HashSet::new())
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
        // Return default if file doesn't exist
        if !path.exists() {
            return Ok(Self::default());
        }

        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        // Cycle detection
        if !seen.insert(canonical.clone()) {
            return Err(ConfigError::CyclicInclude(canonical));
        }

        let content = fs::read_to_string(path)?;
        let (sources, toml_content) = extract_sources(&content);
        let mut config: Self = toml::from_str(&toml_content)?;

        // Process sources and merge (main config takes precedence)
        for source_path in sources {
            let expanded = shellexpand::tilde(&source_path);
            let source_file = Path::new(expanded.as_ref());
            if source_file.exists() {
                let source_config = Self::load_with_sources(source_file, seen)?;
                config.merge(source_config);
            }
        }

        Ok(config)
    }

    /// Merges another config into self.
    ///
    /// Values from `other` are only used if not already set in `self`.
    /// `HashMap` fields are merged (self's values take precedence).
    pub fn merge(&mut self, other: Self) {
        // Merge HashMaps (other's values, then self overwrites)
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

    /// Returns the default config file path (`~/.config/hypr/hyprlog.conf`).
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = Config::default();
        assert_eq!(config.general.level, "info");
        assert!(config.terminal.enabled);
        assert!(!config.file.enabled);
    }

    #[test]
    fn parse_level() {
        let mut config = Config::default();
        config.general.level = "debug".to_string();
        assert_eq!(config.parse_level(), Level::Debug);
    }

    #[test]
    fn parse_toml() {
        let toml = r#"
[general]
level = "debug"
app_name = "testapp"

[terminal]
enabled = true
colors = false
icons = "ascii"

[tag]
prefix = "<"
suffix = ">"
transform = "lowercase"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.general.level, "debug");
        assert_eq!(config.general.app_name, "testapp");
        assert!(!config.terminal.colors);
        assert_eq!(config.tag.prefix, "<");
        assert_eq!(config.parse_transform(), Transform::Lowercase);
    }

    #[test]
    fn parse_colors() {
        let toml = r##"
[colors]
red = "#ff0000"
green = "#00ff00"
"##;
        let config: Config = toml::from_str(toml).unwrap();
        let red = config.get_color("red").unwrap();
        assert_eq!(red.r, 255);
        assert_eq!(red.g, 0);
    }

    #[test]
    fn parse_presets() {
        let toml = r#"
[presets.startup]
level = "info"
scope = "INIT"
msg = "Application started"

[presets.shutdown]
level = "info"
scope = "INIT"
msg = "Application stopped"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.presets.len(), 2);
        assert_eq!(config.presets["startup"].scope, Some("INIT".to_string()));
    }
}
