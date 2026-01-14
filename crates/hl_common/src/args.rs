//! Shared CLI argument types.

use clap::{Args, ValueEnum};
use hl_core::Level;

/// Log level argument with clap integration.
#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => Self::Trace,
            LogLevel::Debug => Self::Debug,
            LogLevel::Info => Self::Info,
            LogLevel::Warn => Self::Warn,
            LogLevel::Error => Self::Error,
        }
    }
}

/// Common log command arguments.
#[derive(Debug, Args)]
pub struct LogArgs {
    /// Log level.
    #[arg(value_enum)]
    pub level: LogLevel,

    /// Scope/component name.
    pub scope: String,

    /// Log message.
    pub message: String,
}

/// Size argument parser for cleanup commands.
#[derive(Debug, Clone)]
pub struct SizeArg(pub u64);

impl std::str::FromStr for SizeArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hl_core::parse_size(s)
            .map(Self)
            .ok_or_else(|| format!("invalid size: {s}"))
    }
}

/// Age argument parser (e.g., "30d", "7d").
#[derive(Debug, Clone)]
pub struct AgeArg(pub u32);

impl std::str::FromStr for AgeArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();
        let days = if let Some(num) = s.strip_suffix('d') {
            num.parse::<u32>().map_err(|e| e.to_string())?
        } else {
            s.parse::<u32>().map_err(|e| e.to_string())?
        };
        Ok(Self(days))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_level_conversion() {
        assert_eq!(Level::from(LogLevel::Debug), Level::Debug);
        assert_eq!(Level::from(LogLevel::Error), Level::Error);
    }

    #[test]
    fn size_arg_parsing() {
        assert_eq!("500M".parse::<SizeArg>().unwrap().0, 500 * 1024 * 1024);
        assert_eq!("1G".parse::<SizeArg>().unwrap().0, 1024 * 1024 * 1024);
        assert!("invalid".parse::<SizeArg>().is_err());
    }

    #[test]
    fn age_arg_parsing() {
        assert_eq!("30d".parse::<AgeArg>().unwrap().0, 30);
        assert_eq!("7".parse::<AgeArg>().unwrap().0, 7);
        assert!("invalid".parse::<AgeArg>().is_err());
    }
}
