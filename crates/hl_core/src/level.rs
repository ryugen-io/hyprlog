//! Log level definitions.

use std::fmt;
use std::str::FromStr;

/// Log severity levels, ordered from most to least verbose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Level {
    /// Fine-grained debugging information.
    Trace = 0,
    /// Debugging information.
    Debug = 1,
    /// Informational messages.
    #[default]
    Info = 2,
    /// Warning messages.
    Warn = 3,
    /// Error messages.
    Error = 4,
}

impl Level {
    /// Returns the canonical lowercase name.
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

    /// Returns all levels in order of verbosity.
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

/// Error returned when parsing an invalid level string.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_ordering() {
        assert!(Level::Trace < Level::Debug);
        assert!(Level::Debug < Level::Info);
        assert!(Level::Info < Level::Warn);
        assert!(Level::Warn < Level::Error);
    }

    #[test]
    fn level_display() {
        assert_eq!(Level::Trace.to_string(), "trace");
        assert_eq!(Level::Debug.to_string(), "debug");
        assert_eq!(Level::Info.to_string(), "info");
        assert_eq!(Level::Warn.to_string(), "warn");
        assert_eq!(Level::Error.to_string(), "error");
    }

    #[test]
    fn level_from_str() {
        assert_eq!("trace".parse::<Level>().unwrap(), Level::Trace);
        assert_eq!("DEBUG".parse::<Level>().unwrap(), Level::Debug);
        assert_eq!("Info".parse::<Level>().unwrap(), Level::Info);
        assert_eq!("warning".parse::<Level>().unwrap(), Level::Warn);
        assert_eq!("err".parse::<Level>().unwrap(), Level::Error);
    }

    #[test]
    fn level_from_str_invalid() {
        assert!("invalid".parse::<Level>().is_err());
    }

    #[test]
    fn level_default() {
        assert_eq!(Level::default(), Level::Info);
    }
}
