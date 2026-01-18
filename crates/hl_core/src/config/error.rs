//! Configuration error types.

use std::path::PathBuf;

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
