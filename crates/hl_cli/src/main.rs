//! hyprlog - Log messages from the command line.
//!
//! Usage:
//!   hyprlog                              Enter interactive shell
//!   hyprlog log <level> <scope> <msg>    Log a message
//!   hyprlog <level> <scope> <msg>        Shorthand logging
//!   hyprlog json [<json>]                Log from JSON (or stdin)
//!   hyprlog preset <name>                Run a preset
//!   hyprlog presets                      List presets
//!   hyprlog stats                        Show statistics
//!   hyprlog cleanup [options]            Clean up logs
//!   hyprlog help                         Show help

use hl_common::{OutputFormatter, PresetRunner};
use hl_core::{CleanupOptions, Config, IconSet, Level, Logger, cleanup, internal, stats};
use serde::Deserialize;
use std::io::{self, BufRead};
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Load config
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            // Can't use internal logger yet - no config
            eprintln!("Error loading config: {e}");
            return ExitCode::FAILURE;
        }
    };

    // Init internal logging with config
    internal::init_with_config(&config);

    // No args = shell mode
    if args.is_empty() {
        return match hl_shell::run(&config) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                internal::error("SHELL", &format!("Shell error: {e}"));
                ExitCode::FAILURE
            }
        };
    }

    // Build logger for commands
    let logger = build_logger(&config, None);
    let args_str: Vec<&str> = args.iter().map(String::as_str).collect();

    match args_str[0] {
        "help" | "--help" | "-h" => {
            print_help();
            ExitCode::SUCCESS
        }
        "version" | "--version" | "-V" => {
            println!("hyprlog {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        "log" => cmd_log(&args_str[1..], &logger),
        "json" => cmd_json(args_str.get(1).copied(), &logger),
        "preset" => cmd_preset(&args_str[1..], &config, &logger),
        "presets" => cmd_presets(&config, &logger),
        "stats" => cmd_stats(&args_str[1..], &config),
        "cleanup" => cmd_cleanup(&args_str[1..], &config),
        // Shorthand: hyprlog info SCOPE msg
        "trace" | "debug" | "info" | "warn" | "error" => cmd_log_shorthand(&args_str, &logger),
        _ => {
            internal::error("CLI", &format!("Unknown command: {}", args_str[0]));
            internal::info("CLI", "Run 'hyprlog help' for usage");
            ExitCode::FAILURE
        }
    }
}

fn cmd_log(args: &[&str], logger: &Logger) -> ExitCode {
    if args.len() < 3 {
        internal::warn("CLI", "Usage: hyprlog log <level> <scope> <message>");
        return ExitCode::FAILURE;
    }
    let Some(level) = parse_level(args[0]) else {
        internal::error("CLI", &format!("Invalid level: {}", args[0]));
        return ExitCode::FAILURE;
    };
    logger.log(level, args[1], &args[2..].join(" "));
    ExitCode::SUCCESS
}

fn cmd_log_shorthand(args: &[&str], logger: &Logger) -> ExitCode {
    if args.len() < 3 {
        internal::warn("CLI", "Usage: hyprlog <level> <scope> <message>");
        return ExitCode::FAILURE;
    }
    let Some(level) = parse_level(args[0]) else {
        internal::error("CLI", &format!("Invalid level: {}", args[0]));
        return ExitCode::FAILURE;
    };
    logger.log(level, args[1], &args[2..].join(" "));
    ExitCode::SUCCESS
}

/// JSON log entry format.
#[derive(Debug, Deserialize)]
struct JsonLogEntry {
    level: String,
    scope: String,
    msg: String,
}

fn cmd_json(input: Option<&str>, logger: &Logger) -> ExitCode {
    let mut processed = 0u64;
    let mut failed = 0u64;

    let process_line = |line: &str| -> Result<(), String> {
        let entry: JsonLogEntry =
            serde_json::from_str(line).map_err(|e| format!("invalid JSON: {e}"))?;

        let level =
            parse_level(&entry.level).ok_or_else(|| format!("invalid level: {}", entry.level))?;

        logger.log(level, &entry.scope, &entry.msg);
        Ok(())
    };

    match input {
        None | Some("-") => {
            internal::debug("JSON", "Reading JSON from stdin");
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                match line {
                    Ok(l) if !l.trim().is_empty() => {
                        internal::trace("JSON", "Processing JSON line");
                        if let Err(e) = process_line(&l) {
                            internal::error("JSON", &e);
                            failed += 1;
                        } else {
                            processed += 1;
                        }
                    }
                    Ok(_) => {}
                    Err(e) => {
                        internal::error("JSON", &format!("Error reading stdin: {e}"));
                        return ExitCode::FAILURE;
                    }
                }
            }
            internal::info(
                "JSON",
                &format!("JSON: processed {processed} entries, {failed} failed"),
            );
            ExitCode::SUCCESS
        }
        Some(json) => {
            internal::trace("JSON", "Processing JSON line");
            match process_line(json) {
                Ok(()) => {
                    internal::info("JSON", "JSON: processed 1 entry, 0 failed");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    internal::error("JSON", &e);
                    internal::info("JSON", "JSON: processed 0 entries, 1 failed");
                    ExitCode::FAILURE
                }
            }
        }
    }
}

fn cmd_preset(args: &[&str], config: &Config, logger: &Logger) -> ExitCode {
    if args.is_empty() {
        internal::warn("CLI", "Usage: hyprlog preset <name>");
        return ExitCode::FAILURE;
    }
    let runner = PresetRunner::new(config, logger);
    match runner.run(args[0]) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            internal::error("PRESET", &format!("{e}"));
            ExitCode::FAILURE
        }
    }
}

fn cmd_presets(config: &Config, logger: &Logger) -> ExitCode {
    let runner = PresetRunner::new(config, logger);
    let list = runner.list();
    if list.is_empty() {
        println!("No presets defined");
    } else {
        println!("Available presets:");
        for name in list {
            println!("  {name}");
        }
    }
    ExitCode::SUCCESS
}

fn cmd_stats(args: &[&str], config: &Config) -> ExitCode {
    let base_dir = expand_path(&config.file.base_dir);

    // Parse --app filter
    let app_filter = args
        .iter()
        .position(|&a| a == "--app")
        .and_then(|i| args.get(i + 1).copied());

    match stats(&base_dir, app_filter) {
        Ok(s) => {
            let formatter = OutputFormatter::new().colors(config.terminal.colors);
            println!("{}", formatter.format_stats(&s));
            ExitCode::SUCCESS
        }
        Err(e) => {
            internal::error("STATS", &format!("{e}"));
            ExitCode::FAILURE
        }
    }
}

fn cmd_cleanup(args: &[&str], config: &Config) -> ExitCode {
    let dry_run = args.contains(&"--dry-run");
    let all = args.contains(&"--all");
    let compress = args.contains(&"--compress");

    internal::debug(
        "CLEANUP",
        &format!("dry_run={dry_run}, all={all}, compress={compress}"),
    );

    let mut options = CleanupOptions::new()
        .dry_run(dry_run)
        .delete_all(all)
        .compress(compress);

    // Apply config defaults first
    if let Some(days) = config.cleanup.max_age_days {
        internal::debug("CLEANUP", &format!("Config: max_age_days={days}"));
        options = options.max_age_days(days);
    }
    if let Some(ref size) = config.cleanup.max_total_size {
        internal::debug("CLEANUP", &format!("Config: max_total_size={size}"));
        options = options.max_total_size(size);
    }
    if let Some(keep) = config.cleanup.keep_last {
        internal::debug("CLEANUP", &format!("Config: keep_last={keep}"));
        options = options.keep_last(keep);
    }

    // CLI overrides config
    if let Some(idx) = args.iter().position(|&a| a == "--older-than") {
        if let Some(days_str) = args.get(idx + 1) {
            if let Ok(days) = days_str.trim_end_matches('d').parse::<u32>() {
                internal::debug("CLEANUP", &format!("CLI override: max_age_days={days}"));
                options = options.max_age_days(days);
            }
        }
    }

    if let Some(idx) = args.iter().position(|&a| a == "--max-size") {
        if let Some(size_str) = args.get(idx + 1) {
            internal::debug("CLEANUP", &format!("CLI override: max_size={size_str}"));
            options = options.max_total_size(size_str);
        }
    }

    if let Some(idx) = args.iter().position(|&a| a == "--app") {
        if let Some(app) = args.get(idx + 1) {
            internal::debug("CLEANUP", &format!("CLI override: app={app}"));
            options = options.app_filter((*app).to_string());
        }
    }

    if let Some(idx) = args.iter().position(|&a| a == "--keep-last") {
        if let Some(n_str) = args.get(idx + 1) {
            if let Ok(n) = n_str.parse::<usize>() {
                internal::debug("CLEANUP", &format!("CLI override: keep_last={n}"));
                options = options.keep_last(n);
            }
        }
    }

    if let Some(idx) = args.iter().position(|&a| a == "--before") {
        if let Some(date_str) = args.get(idx + 1) {
            if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                internal::debug("CLEANUP", &format!("CLI override: before={date}"));
                options = options.before_date(date);
            } else {
                internal::error(
                    "CLEANUP",
                    &format!("Invalid date format for --before: {date_str} (use YYYY-MM-DD)"),
                );
                return ExitCode::FAILURE;
            }
        }
    }

    if let Some(idx) = args.iter().position(|&a| a == "--after") {
        if let Some(date_str) = args.get(idx + 1) {
            if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                internal::debug("CLEANUP", &format!("CLI override: after={date}"));
                options = options.after_date(date);
            } else {
                internal::error(
                    "CLEANUP",
                    &format!("Invalid date format for --after: {date_str} (use YYYY-MM-DD)"),
                );
                return ExitCode::FAILURE;
            }
        }
    }

    let base_dir = expand_path(&config.file.base_dir);
    internal::debug("CLEANUP", &format!("Base dir: {}", base_dir.display()));

    match cleanup(&base_dir, &options) {
        Ok(result) => {
            // Log failures
            for (path, err) in &result.failed {
                internal::warn("CLEANUP", &format!("Failed to process {path}: {err}"));
            }

            let formatter = OutputFormatter::new().colors(config.terminal.colors);
            println!("{}", formatter.format_cleanup(&result, dry_run));
            ExitCode::SUCCESS
        }
        Err(e) => {
            internal::error("CLEANUP", &format!("{e}"));
            ExitCode::FAILURE
        }
    }
}

fn parse_level(s: &str) -> Option<Level> {
    match s.to_lowercase().as_str() {
        "trace" => Some(Level::Trace),
        "debug" => Some(Level::Debug),
        "info" => Some(Level::Info),
        "warn" => Some(Level::Warn),
        "error" => Some(Level::Error),
        _ => None,
    }
}

fn print_help() {
    println!(
        "hyprlog - Flexible logging from the command line

Usage:
  hyprlog                              Enter interactive shell
  hyprlog log <level> <scope> <msg>    Log a message
  hyprlog <level> <scope> <msg>        Shorthand logging
  hyprlog json [<json>]                Log from JSON (or stdin with -)
  hyprlog preset <name>                Run a preset
  hyprlog presets                      List available presets
  hyprlog stats [--app <name>]         Show log statistics
  hyprlog cleanup [options]            Clean up old logs
    --older-than <N>d                  Delete files older than N days
    --before <DATE>                    Delete files modified before DATE (YYYY-MM-DD)
    --after <DATE>                     Delete files modified after DATE (YYYY-MM-DD)
    --max-size <size>                  Keep total size under limit (e.g., 500M, 1G)
    --keep-last <N>                    Always keep the N most recent files
    --compress                         Compress files (gzip) instead of deleting
    --app <name>                       Filter by app name
    --all                              Delete all files
    --dry-run                          Show what would be done
  hyprlog help                         Show this help
  hyprlog version                      Show version

Config defaults (in ~/.config/hypr/hyprlog.conf):
  [cleanup]
  max_age_days = 30
  max_total_size = \"500M\"
  keep_last = 5

Levels: trace, debug, info, warn, error

Examples:
  hyprlog info INIT \"Application started\"
  hyprlog log error NET \"Connection failed\"
  hyprlog cleanup --dry-run
  hyprlog cleanup --compress --older-than 7d --keep-last 5
  hyprlog cleanup --before 2024-01-01 --dry-run
  echo '{{\"level\":\"info\",\"scope\":\"TEST\",\"msg\":\"hello\"}}' | hyprlog json"
    );
}

fn build_logger(config: &Config, app_override: Option<&str>) -> Logger {
    let app_name = app_override
        .or(config.general.app_name.as_deref())
        .unwrap_or("hyprlog");
    let mut builder = Logger::builder().level(config.parse_level());

    if config.terminal.enabled {
        builder = builder
            .terminal()
            .colors(config.terminal.colors)
            .icons(IconSet::from(config.parse_icon_type()))
            .structure(&config.terminal.structure)
            .done();
    }

    if config.file.enabled {
        builder = builder
            .file()
            .base_dir(&config.file.base_dir)
            .path_structure(&config.file.path_structure)
            .filename_structure(&config.file.filename_structure)
            .content_structure(&config.file.content_structure)
            .timestamp_format(&config.file.timestamp_format)
            .app_name(app_name)
            .done();
    }

    builder.build()
}

fn expand_path(path: &str) -> PathBuf {
    if path.starts_with('~') {
        if let Some(user_dirs) = directories::UserDirs::new() {
            return PathBuf::from(path.replacen(
                '~',
                user_dirs.home_dir().to_str().unwrap_or(""),
                1,
            ));
        }
    }
    PathBuf::from(path)
}
