//! CLI module for hyprlog.
//!
//! This module provides the command-line interface using Clap.

pub mod commands;
pub mod preset;
pub mod util;

use clap::{Parser, Subcommand};

/// Log level for CLI arguments.
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for crate::level::Level {
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

/// Theme action for the themes subcommand.
#[derive(Debug, Clone, Copy, clap::ValueEnum, Default)]
pub enum ThemeAction {
    #[default]
    List,
    Preview,
}

/// hyprlog - Log messages from the command line.
#[derive(Parser)]
#[command(
    name = "hyprlog",
    version,
    about = "Log messages from the command line"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// CLI subcommands.
#[derive(Subcommand)]
pub enum Command {
    /// Log a message with explicit app name.
    Log {
        /// Application name
        app: String,
        /// Log level
        #[arg(value_enum)]
        level: LogLevel,
        /// Scope/component name
        scope: String,
        /// Log message
        message: Vec<String>,
    },
    /// Log from JSON input.
    Json {
        /// JSON string (reads stdin if omitted or "-")
        json: Option<String>,
    },
    /// Run a preset.
    Preset {
        /// Preset name
        name: String,
    },
    /// List available presets.
    Presets,
    /// Show log statistics.
    Stats {
        /// Filter by app name
        #[arg(short, long)]
        app: Option<String>,
    },
    /// Clean up old logs.
    Cleanup {
        /// Show what would be done without doing it
        #[arg(long)]
        dry_run: bool,
        /// Delete all files
        #[arg(long)]
        all: bool,
        /// Delete files older than N days (e.g., "30d" or "30")
        #[arg(long, value_name = "DAYS")]
        older_than: Option<String>,
        /// Keep total size under limit (e.g., "500M", "1G")
        #[arg(long, value_name = "SIZE")]
        max_size: Option<String>,
        /// Always keep the N most recent files
        #[arg(long, value_name = "N")]
        keep_last: Option<usize>,
        /// Filter by app name
        #[arg(long)]
        app: Option<String>,
        /// Compress files instead of deleting
        #[arg(long)]
        compress: bool,
        /// Delete files modified before DATE (YYYY-MM-DD)
        #[arg(long, value_name = "DATE")]
        before: Option<String>,
        /// Delete files modified after DATE (YYYY-MM-DD)
        #[arg(long, value_name = "DATE")]
        after: Option<String>,
    },
    /// List or preview prompt themes.
    Themes {
        /// Action to perform
        #[arg(value_enum, default_value = "list")]
        action: ThemeAction,
    },
}

#[cfg(feature = "hyprland")]
pub use commands::cmd_watch;
pub use commands::{
    cmd_cleanup, cmd_json, cmd_log, cmd_log_shorthand, cmd_preset, cmd_presets, cmd_stats,
    cmd_themes,
};
pub use preset::PresetRunner;
pub use util::{build_logger, expand_path, parse_level, print_help};
