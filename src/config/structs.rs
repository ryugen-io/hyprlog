//! Configuration struct definitions.

use serde::Deserialize;
use std::collections::HashMap;

/// General configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    /// Minimum log level.
    pub level: String,
    /// Application name.
    pub app_name: Option<String>,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            app_name: None,
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

/// Shell configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ShellConfig {
    /// Prompt theme.
    pub theme: String,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            theme: "dracula".to_string(),
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
        let base_dir = directories::ProjectDirs::from("", "", "hyprlog").map_or_else(
            || "logs".to_string(),
            |dirs| {
                dirs.state_dir()
                    .unwrap_or_else(|| dirs.data_dir())
                    .join("logs")
                    .to_string_lossy()
                    .into_owned()
            },
        );

        Self {
            enabled: false,
            base_dir,
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

/// Cleanup configuration defaults.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct CleanupConfig {
    /// Maximum age in days (None = no age limit).
    pub max_age_days: Option<u32>,
    /// Maximum total size (e.g., "500M", "1G").
    pub max_total_size: Option<String>,
    /// Always keep the N most recent files.
    pub keep_last: Option<usize>,
    /// Compress files older than N days instead of deleting.
    pub compress_after_days: Option<u32>,
}

/// Message formatting configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct MessageConfigFile {
    /// Text transform (none, uppercase, lowercase, capitalize).
    pub transform: String,
}

impl Default for MessageConfigFile {
    fn default() -> Self {
        Self {
            transform: "none".to_string(),
        }
    }
}

/// Scope formatting configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ScopeConfigFile {
    /// Minimum width (padded if shorter).
    pub min_width: usize,
    /// Alignment (left, right, center).
    pub alignment: String,
    /// Text transform (none, uppercase, lowercase, capitalize).
    pub transform: String,
}

impl Default for ScopeConfigFile {
    fn default() -> Self {
        Self {
            min_width: 12,
            alignment: "left".to_string(),
            transform: "none".to_string(),
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
    /// Display label (shown in output).
    pub level: String,
    /// Internal level for filtering (e.g., "info" when level is "success").
    #[serde(default, rename = "as")]
    pub as_level: Option<String>,
    /// Scope.
    pub scope: Option<String>,
    /// Message.
    pub msg: String,
    /// Application name override.
    pub app_name: Option<String>,
}

/// Auto-highlighting configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct HighlightConfig {
    /// Enable auto-highlighting.
    pub enabled: bool,
    /// Keywords to highlight (keyword -> color name).
    pub keywords: HashMap<String, String>,
    /// Pattern-based highlighting.
    pub patterns: PatternsConfig,
}

impl Default for HighlightConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            keywords: HashMap::new(),
            patterns: PatternsConfig::default(),
        }
    }
}

/// Pattern-based highlighting configuration.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct PatternsConfig {
    /// Color for file paths (/path/to/file, ./relative, ~/home).
    pub paths: Option<String>,
    /// Color for URLs (https://..., http://...).
    pub urls: Option<String>,
    /// Color for numbers (123, 3.14, -42).
    pub numbers: Option<String>,
    /// Color for quoted strings ("string" or 'string').
    pub quoted: Option<String>,
}

/// Hyprland IPC integration configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct HyprlandConfig {
    /// Enable Hyprland IPC integration.
    pub enabled: bool,
    /// Override `HYPRLAND_INSTANCE_SIGNATURE` for socket path resolution.
    pub instance_signature: Option<String>,
    /// Override the socket directory path directly.
    pub socket_dir: Option<String>,
    /// Per-event log level overrides (event name -> level string).
    pub event_levels: HashMap<String, String>,
    /// Events to ignore entirely.
    pub ignore_events: Vec<String>,
    /// Scope string used for Hyprland log messages.
    pub scope: String,
    /// Per-event scope overrides (event name -> scope string).
    pub event_scopes: HashMap<String, String>,
    /// Use human-readable event formatting (default: true).
    /// When false, events are logged with raw Hyprland wire format.
    pub human_readable: bool,
    /// Runtime-only allowlist filter (not deserialized from config).
    /// When set, only events in this list are processed.
    #[serde(skip)]
    pub event_filter: Option<Vec<String>>,
}

impl Default for HyprlandConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            instance_signature: None,
            socket_dir: None,
            event_levels: HashMap::new(),
            ignore_events: Vec::new(),
            scope: "hyprland".to_string(),
            event_scopes: HashMap::new(),
            human_readable: true,
            event_filter: None,
        }
    }
}

/// Per-app configuration overrides.
///
/// Used in `[apps.X]` sections to override global settings for specific apps.
/// All fields are optional - only specified fields override the global config.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct AppConfig {
    /// Override log level for this app.
    pub level: Option<String>,
    /// Override terminal settings.
    pub terminal: Option<AppTerminalConfig>,
    /// Override file settings.
    pub file: Option<AppFileConfig>,
}

/// Per-app terminal overrides.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct AppTerminalConfig {
    /// Override enabled state.
    pub enabled: Option<bool>,
    /// Override colors.
    pub colors: Option<bool>,
    /// Override icons.
    pub icons: Option<String>,
    /// Override structure template.
    pub structure: Option<String>,
}

/// Per-app file output overrides.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct AppFileConfig {
    /// Override enabled state.
    pub enabled: Option<bool>,
    /// Override base directory.
    pub base_dir: Option<String>,
}

/// JSON database output configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct JsonConfig {
    /// Enable JSON database output.
    pub enabled: bool,
    /// Path to the JSONL database file.
    pub path: String,
}

impl Default for JsonConfig {
    fn default() -> Self {
        let path = directories::ProjectDirs::from("", "", "hyprlog").map_or_else(
            || "hyprlog.jsonl".to_string(),
            |dirs| {
                dirs.state_dir()
                    .unwrap_or_else(|| dirs.data_dir())
                    .join("db")
                    .join("hyprlog.jsonl")
                    .to_string_lossy()
                    .into_owned()
            },
        );

        Self {
            enabled: false,
            path,
        }
    }
}
