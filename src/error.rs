//! Unified error type for all hyprlog operations.

use std::path::PathBuf;

/// Error type for hyprlog operations.
#[derive(Debug)]
pub enum Error {
    /// I/O error.
    Io(std::io::Error),
    /// TOML config parsing error.
    ConfigParse(toml::de::Error),
    /// Config directory not found.
    ConfigDirNotFound,
    /// Cyclic include detected in config sources.
    CyclicInclude(PathBuf),
    /// Format/serialization error.
    Format(String),
    /// Invalid path.
    InvalidPath(String),
    /// Preset not found.
    PresetNotFound(String),
    /// Invalid log level string.
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
