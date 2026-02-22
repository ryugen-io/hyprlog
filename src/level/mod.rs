//! Severity levels that gate which messages reach which outputs.

use std::fmt;
use std::str::FromStr;

/// Derives `Ord` so the logger can compare a message's level against the configured minimum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Level {
    /// High-volume instrumentation that would be too noisy outside of development.
    Trace = 0,
    /// Startup, teardown, and state-change details useful for diagnosing issues.
    Debug = 1,
    /// Normal operational milestones — connection established, config loaded, etc.
    #[default]
    Info = 2,
    /// Non-fatal anomalies that may need attention (deprecated features, retries).
    Warn = 3,
    /// Unrecoverable failures that prevent the operation from completing.
    Error = 4,
}

impl Level {
    /// Lowercase because config files and CLI args use lowercase level strings.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }

    /// Convenience for iteration — used by help output, shell completion, and tests.
    #[must_use]
    pub const fn all() -> [Self; 5] {
        [
            Self::Trace,
            Self::Debug,
            Self::Info,
            Self::Warn,
            Self::Error,
        ]
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Returned by `FromStr` so callers can distinguish "unknown level" from other parse failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseLevelError(String);

impl fmt::Display for ParseLevelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown log level: '{}'", self.0)
    }
}

impl std::error::Error for ParseLevelError {}

impl FromStr for Level {
    type Err = ParseLevelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "trace" => Ok(Self::Trace),
            "debug" => Ok(Self::Debug),
            "info" => Ok(Self::Info),
            "warn" | "warning" => Ok(Self::Warn),
            "error" | "err" => Ok(Self::Error),
            _ => Err(ParseLevelError(s.to_string())),
        }
    }
}
