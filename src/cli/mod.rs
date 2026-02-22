//! Without a CLI, every hyprlog operation would require Rust code or the REPL.
//!
//! The Clap-based interface lets shell scripts, systemd units, and one-liners
//! emit structured logs without touching the library API.

pub mod commands;
pub mod preset;
pub mod util;

use clap::{Parser, Subcommand};

/// Clap needs its own enum for `value_enum` derive — the library's Level type isn't Clap-aware.
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

/// Separate enum avoids overloading the theme name argument with subcommand semantics.
#[derive(Debug, Clone, Copy, clap::ValueEnum, Default)]
pub enum ThemeAction {
    #[default]
    List,
    Preview,
}

/// Single entry point for all CLI invocations — Clap derives help, version, and dispatch from this struct.
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

/// Without subcommands, every operation would need separate flags — this keeps the interface discoverable.
#[derive(Subcommand)]
pub enum Command {
    /// Emit a structured log entry from the command line — useful for shell scripts and automation.
    Log {
        /// Without an app name, all entries share one output directory and miss per-app config overrides.
        app: String,
        /// Without severity, every message would be treated equally — no way to silence debug noise in production.
        #[arg(value_enum)]
        level: LogLevel,
        /// Dense logs from multiple subsystems are unreadable without a scope to grep/filter on.
        scope: String,
        /// Vec collects all remaining args — avoids requiring shell quoting for multi-word messages.
        message: Vec<String>,
    },
    /// Accepts pre-structured log data — integrations that already have JSON don't need to decompose it into positional args.
    Json {
        /// Stdin fallback lets piped workflows pass JSON without a shell argument.
        json: Option<String>,
    },
    /// Presets bundle multiple log operations into a single name — saves typing repetitive commands.
    Preset {
        /// Must match a key in the config's `[presets]` table — unknown names fail with an error.
        name: String,
    },
    /// Users need to discover what presets exist before they can run one.
    Presets,
    /// Disk usage and log volume are invisible without explicit reporting — stats surface them.
    Stats {
        /// Without filtering, stats aggregate across all apps — useless for per-app disk analysis.
        #[arg(short, long)]
        app: Option<String>,
    },
    /// Log files grow unbounded without manual intervention — cleanup automates retention policies.
    Cleanup {
        /// Destructive operations need a preview mode — users want to verify before deleting files.
        #[arg(long)]
        dry_run: bool,
        /// Sometimes a full wipe is the only way to reclaim disk space quickly.
        #[arg(long)]
        all: bool,
        /// Age-based retention keeps recent logs for debugging while discarding stale data.
        #[arg(long, value_name = "DAYS")]
        older_than: Option<String>,
        /// Size-based retention prevents logs from filling the partition regardless of age.
        #[arg(long, value_name = "SIZE")]
        max_size: Option<String>,
        /// Safety net — even aggressive cleanup policies shouldn't delete the most recent logs.
        #[arg(long, value_name = "N")]
        keep_last: Option<usize>,
        /// Multi-app setups need per-app cleanup — without this, one noisy app's logs can't be purged independently.
        #[arg(long)]
        app: Option<String>,
        /// Compression preserves data for future analysis while still reclaiming most of the disk space.
        #[arg(long)]
        compress: bool,
        /// Date-based cutoff gives finer control than day-count when targeting a specific incident window.
        #[arg(long, value_name = "DATE")]
        before: Option<String>,
        /// Paired with `before` to define a date range — useful for purging logs from a known-bad period.
        #[arg(long, value_name = "DATE")]
        after: Option<String>,
    },
    /// Users need to see available themes before committing to one in their config.
    Themes {
        /// "list" shows names only, "preview" renders colored prompts — two levels of detail for different needs.
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
