//! Utility functions for the CLI.

use crate::config::Config;
use crate::level::Level;
use crate::logger::Logger;
use std::path::PathBuf;

/// Parses a level string to a Level enum.
#[must_use]
pub fn parse_level(s: &str) -> Option<Level> {
    match s.to_lowercase().as_str() {
        "trace" => Some(Level::Trace),
        "debug" => Some(Level::Debug),
        "info" => Some(Level::Info),
        "warn" => Some(Level::Warn),
        "error" => Some(Level::Error),
        _ => None,
    }
}

/// Expands a path with tilde to the user's home directory.
#[must_use]
pub fn expand_path(path: &str) -> PathBuf {
    if path.starts_with('~')
        && let Some(user_dirs) = directories::UserDirs::new()
    {
        return PathBuf::from(path.replacen('~', user_dirs.home_dir().to_str().unwrap_or(""), 1));
    }
    PathBuf::from(path)
}

/// Builds a logger from config with optional app name override.
#[must_use]
pub fn build_logger(config: &Config, app_override: Option<&str>) -> Logger {
    let app_name = app_override.unwrap_or("hyprlog");
    Logger::from_config_with(config, app_name)
}

/// Prints the help message.
pub fn print_help() {
    println!(
        "hyprlog - Flexible logging from the command line

Usage:
  hyprlog                                   Enter interactive shell
  hyprlog log <app> <level> <scope> <msg>   Log a message for specific app
  hyprlog [<app>] <level> <scope> <msg>     Shorthand (app defaults to 'hyprlog')
  hyprlog json [<json>]                     Log from JSON (or stdin with -)
  hyprlog preset <name>                     Run a preset
  hyprlog presets                           List available presets
  hyprlog stats [--app <name>]              Show log statistics
  hyprlog themes [list|preview]             List or preview prompt themes
  hyprlog cleanup [options]                 Clean up old logs
    --older-than <N>d                       Delete files older than N days
    --before <DATE>                         Delete files modified before DATE (YYYY-MM-DD)
    --after <DATE>                          Delete files modified after DATE (YYYY-MM-DD)
    --max-size <size>                       Keep total size under limit (e.g., 500M, 1G)
    --keep-last <N>                         Always keep the N most recent files
    --compress                              Compress files (gzip) instead of deleting
    --app <name>                            Filter by app name
    --all                                   Delete all files
    --dry-run                               Show what would be done
  hyprlog help                              Show this help
  hyprlog version                           Show version

Config defaults (in ~/.config/hypr/hyprlog.conf):
  [cleanup]
  max_age_days = 30
  max_total_size = \"500M\"
  keep_last = 5

Levels: trace, debug, info, warn, error

Examples:
  hyprlog info INIT \"Application started\"
  hyprlog myapp info INIT \"Application started\"
  hyprlog log myapp error NET \"Connection failed\"
  hyprlog cleanup --dry-run
  hyprlog cleanup --compress --older-than 7d --keep-last 5
  hyprlog cleanup --before 2024-01-01 --dry-run
  echo '{{\"level\":\"info\",\"scope\":\"TEST\",\"msg\":\"hello\"}}' | hyprlog json"
    );
}
