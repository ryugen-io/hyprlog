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
        let mut keywords = HashMap::new();
        keywords.insert("ERROR".to_string(), "red".to_string());
        keywords.insert("WARN".to_string(), "yellow".to_string());
        keywords.insert("OK".to_string(), "green".to_string());
        keywords.insert("SUCCESS".to_string(), "green".to_string());
        keywords.insert("FAIL".to_string(), "red".to_string());
        keywords.insert("true".to_string(), "green".to_string());
        keywords.insert("false".to_string(), "red".to_string());
        keywords.insert("null".to_string(), "purple".to_string());
        keywords.insert("none".to_string(), "purple".to_string());
        keywords.insert("yes".to_string(), "green".to_string());
        keywords.insert("no".to_string(), "red".to_string());

        Self {
            enabled: true,
            keywords,
            patterns: PatternsConfig::default(),
        }
    }
}

/// Pattern-based highlighting configuration.
#[derive(Debug, Clone, Deserialize)]
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

impl Default for PatternsConfig {
    fn default() -> Self {
        Self {
            paths: Some("cyan".to_string()),
            urls: Some("blue".to_string()),
            numbers: Some("orange".to_string()),
            quoted: Some("yellow".to_string()),
        }
    }
}
