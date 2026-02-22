//! Without a unified error type, every caller would need to handle `io::Error`,
//! `toml::de::Error`, and hyprlog-specific failures separately.

use std::path::PathBuf;

/// Avoids boxing and lets callers `?`-propagate any hyprlog failure through one type.
#[derive(Debug)]
pub enum Error {
    /// File output, socket connections, and directory walks all produce `io::Error`.
    Io(std::io::Error),
    /// Config loading can fail on bad TOML syntax or missing required fields.
    ConfigParse(toml::de::Error),
    /// Some platforms (containers, CI) don't set $HOME or XDG dirs.
    ConfigDirNotFound,
    /// `source = "..."` chains can accidentally form loops without cycle detection.
    CyclicInclude(PathBuf),
    /// Template rendering or JSON serialization can fail on malformed input.
    Format(String),
    /// User-supplied paths from config or CLI may not exist or be accessible.
    InvalidPath(String),
    /// Users can typo preset names in CLI invocations.
    PresetNotFound(String),
    /// Preset configs reference level strings that may not map to a valid severity.
    InvalidLevel(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::ConfigParse(e) => write!(f, "parse error: {e}"),
            Self::ConfigDirNotFound => write!(f, "config directory not found"),
            Self::CyclicInclude(p) => write!(f, "cyclic include: {}", p.display()),
            Self::Format(s) => write!(f, "format error: {s}"),
            Self::InvalidPath(s) => write!(f, "invalid path: {s}"),
            Self::PresetNotFound(name) => write!(f, "preset not found: {name}"),
            Self::InvalidLevel(level) => write!(f, "invalid level in preset: {level}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::ConfigParse(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Self::ConfigParse(e)
    }
}
