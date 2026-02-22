//! Serde schema lives here so config/mod.rs can focus on loading, cycle
//! detection, and merge logic without mixing in struct definitions.

use serde::Deserialize;
use std::collections::HashMap;

/// Severity filtering and app identity apply to all outputs — they belong above any specific backend.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    /// Without severity filtering, every trace message floods all outputs.
    pub level: String,
    /// Multiple apps sharing one config need separate log directories and per-app overrides.
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

/// Terminal is the most common output — users expect immediate stderr feedback without extra setup.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct TerminalConfig {
    /// Off by default would surprise CLI users who expect immediate feedback.
    pub enabled: bool,
    /// Piped output and CI environments can't render ANSI.
    pub colors: bool,
    /// Not every terminal has `NerdFont` glyphs available.
    pub icons: String,
    /// Different projects need different column layouts per log line.
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

/// REPL appearance is independent of logging output — shell users need their own theme control.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ShellConfig {
    /// Users want the REPL to match their terminal aesthetic.
    pub theme: String,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            theme: "dracula".to_string(),
        }
    }
}

/// Persistent logging creates files on disk — it must be opt-in to avoid unexpected disk usage.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct FileConfig {
    /// Disk writes are opt-in — not every use case needs persistent logs.
    pub enabled: bool,
    /// Default XDG path doesn't work for every deployment (containers, custom setups).
    pub base_dir: String,
    /// Different projects organize logs differently (by app, by date, flat).
    pub path_structure: String,
    /// Multiple apps in the same directory need distinct filenames.
    pub filename_structure: String,
    /// File output doesn't need ANSI but may need timestamps or different column order.
    pub content_structure: String,
    /// Different locales and log analysis tools expect different timestamp formats.
    pub timestamp_format: String,
    /// Logs grow forever without automatic rotation.
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

/// Logs grow without bound — age and size limits prevent runaway disk consumption.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct RetentionConfig {
    /// Logs older than this are stale and unlikely to be useful.
    pub max_age_days: u32,
    /// Disk-constrained systems need a hard cap regardless of age.
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

/// Cleanup defaults prevent the subcommand from requiring flags for every common operation.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct CleanupConfig {
    /// Logs older than this are stale and unlikely to be useful.
    pub max_age_days: Option<u32>,
    /// Disk-constrained systems need a hard cap regardless of age.
    pub max_total_size: Option<String>,
    /// Aggressive retention shouldn't delete the most recent diagnostics.
    pub keep_last: Option<usize>,
    /// Compliance needs may require keeping content but not at full size.
    pub compress_after_days: Option<u32>,
}

/// Most log messages need no transformation, but some alert systems require uppercase for visibility.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct MessageConfigFile {
    /// Some alert systems need uppercase messages for visibility.
    pub transform: String,
}

impl Default for MessageConfigFile {
    fn default() -> Self {
        Self {
            transform: "none".to_string(),
        }
    }
}

/// Scope names vary in length and casing across projects — consistent column appearance needs per-project control.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ScopeConfigFile {
    /// Scopes have different lengths — padding keeps columns aligned.
    pub min_width: usize,
    /// Left-aligned scopes are easiest to scan in most terminals.
    pub alignment: String,
    /// Projects may prefer uppercase scopes for visual distinction.
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

/// Level indicators need project-specific delimiters, casing, and width to match each team's log convention.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct TagConfigFile {
    /// Opening delimiter — `[` produces `[INFO]`, `<` produces `<INFO>`.
    pub prefix: String,
    /// Closing delimiter — must pair with prefix for readability.
    pub suffix: String,
    /// Hyprland uses lowercase, most loggers use uppercase — user decides.
    pub transform: String,
    /// Level names have different lengths — padding keeps columns aligned.
    pub min_width: usize,
    /// Centered tags look cleaner with padding; left-aligned are more grep-friendly.
    pub alignment: String,
    /// Projects may want domain-specific names instead of "INFO"/"WARN".
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

/// Built-in glyphs can't cover every preference — per-level overrides let users match their terminal's font.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct IconsConfig {
    /// Users may prefer different glyphs than the built-in defaults.
    pub nerdfont: HashMap<String, String>,
    /// ASCII fallbacks can also be customized per project.
    pub ascii: HashMap<String, String>,
}

/// Repetitive log messages (startup, shutdown, deploy) shouldn't require retyping level, scope, and text every time.
#[derive(Debug, Clone, Deserialize)]
pub struct PresetConfig {
    /// Presets can use custom level names like "success" that aren't real severities.
    pub level: String,
    /// Custom level names need a real severity for filtering (e.g., "success" → "info").
    #[serde(default, rename = "as")]
    pub as_level: Option<String>,
    /// Presets can override the scope so callers don't have to specify it.
    pub scope: Option<String>,
    /// The actual log message — the whole point of having a preset.
    pub msg: String,
    /// Presets can target a specific app's log directory.
    pub app_name: Option<String>,
}

/// Dense log output buries important tokens (paths, URLs, keywords) — color highlighting makes them scannable.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct HighlightConfig {
    /// Highlighting has a runtime cost from regex matching on every message.
    pub enabled: bool,
    /// Domain-specific terms (e.g., "FATAL", "timeout") deserve visual emphasis.
    pub keywords: HashMap<String, String>,
    /// Common patterns (URLs, paths, numbers) are hard to spot in dense output.
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

/// Each pattern type has different visual importance — `None` disables patterns users don't care about.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct PatternsConfig {
    /// File paths are common in log messages and easy to miss without color.
    pub paths: Option<String>,
    /// URLs in logs are often clickable in modern terminals — color helps find them.
    pub urls: Option<String>,
    /// Numeric values (ports, counts, durations) are key diagnostic data.
    pub numbers: Option<String>,
    /// Quoted strings often contain user input or error messages worth highlighting.
    pub quoted: Option<String>,
}

/// IPC event listening only makes sense on Hyprland systems — it must be opt-in to avoid errors elsewhere.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct HyprlandConfig {
    /// IPC listening is opt-in because it only makes sense on Hyprland systems.
    pub enabled: bool,
    /// Containers or nested sessions may need a different instance signature.
    pub instance_signature: Option<String>,
    /// Non-standard Hyprland installs may place sockets in a custom directory.
    pub socket_dir: Option<String>,
    /// Default event levels (Info for most) may be too noisy or too quiet for some events.
    pub event_levels: HashMap<String, String>,
    /// Some events (e.g., mouse moves) fire too frequently to be useful in logs.
    pub ignore_events: Vec<String>,
    /// Users may want IPC events under a different scope than "hyprland".
    pub scope: String,
    /// Different events may belong to different logical scopes (e.g., "window", "workspace").
    pub event_scopes: HashMap<String, String>,
    /// Raw wire format is useful for debugging hyprlog itself but not for end users.
    pub human_readable: bool,
    /// CLI --events flag sets this at runtime — not persisted in config.
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

/// `[apps.X]` sections — different binaries sharing one config file need to diverge.
///
/// All fields are optional — only specified fields override the global config.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct AppConfig {
    /// A debug tool and a production daemon shouldn't share a log level.
    pub level: Option<String>,
    /// Some apps need colors off while others benefit from them.
    pub terminal: Option<AppTerminalConfig>,
    /// Some apps need their own log directory or different file structure.
    pub file: Option<AppFileConfig>,
}

/// Per-app terminal settings must be optional — unset fields should inherit from global, not reset to defaults.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct AppTerminalConfig {
    /// Some apps should be silent on the terminal.
    pub enabled: Option<bool>,
    /// A TUI app can't have ANSI log output mixed into its display.
    pub colors: Option<bool>,
    /// A headless daemon doesn't benefit from `NerdFont` glyphs.
    pub icons: Option<String>,
    /// Different apps may need different column layouts.
    pub structure: Option<String>,
}

/// Per-app file settings must be optional — unset fields should inherit from global, not reset to defaults.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct AppFileConfig {
    /// Some apps should log to disk while others shouldn't.
    pub enabled: Option<bool>,
    /// Apps may need isolated log directories for security or organization.
    pub base_dir: Option<String>,
}

/// Stats queries need structured data — plain log files can't be efficiently queried for aggregates.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct JsonConfig {
    /// JSONL output is opt-in because it duplicates data and grows without bound.
    pub enabled: bool,
    /// Default XDG path doesn't work for every deployment.
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
