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
use hl_core::{CleanupOptions, Config, IconSet, Level, Logger, cleanup, stats};
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
            eprintln!("Error loading config: {e}");
            return ExitCode::FAILURE;
        }
    };

    // No args = shell mode
    if args.is_empty() {
        return match hl_shell::run(&config) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("Error: {e}");
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
            eprintln!("Unknown command: {}", args_str[0]);
            eprintln!("Run 'hyprlog help' for usage");
            ExitCode::FAILURE
        }
    }
}

fn cmd_log(args: &[&str], logger: &Logger) -> ExitCode {
    if args.len() < 3 {
        eprintln!("Usage: hyprlog log <level> <scope> <message>");
        return ExitCode::FAILURE;
    }
    let Some(level) = parse_level(args[0]) else {
        eprintln!("Invalid level: {}", args[0]);
        return ExitCode::FAILURE;
    };
    logger.log(level, args[1], &args[2..].join(" "));
    ExitCode::SUCCESS
}

fn cmd_log_shorthand(args: &[&str], logger: &Logger) -> ExitCode {
    if args.len() < 3 {
        eprintln!("Usage: hyprlog <level> <scope> <message>");
        return ExitCode::FAILURE;
    }
    let Some(level) = parse_level(args[0]) else {
        eprintln!("Invalid level: {}", args[0]);
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
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                match line {
                    Ok(l) if !l.trim().is_empty() => {
                        if let Err(e) = process_line(&l) {
                            eprintln!("Error: {e}");
                        }
                    }
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error reading stdin: {e}");
                        return ExitCode::FAILURE;
                    }
                }
            }
            ExitCode::SUCCESS
        }
        Some(json) => match process_line(json) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("Error: {e}");
                ExitCode::FAILURE
            }
        },
    }
}

fn cmd_preset(args: &[&str], config: &Config, logger: &Logger) -> ExitCode {
    if args.is_empty() {
        eprintln!("Usage: hyprlog preset <name>");
        return ExitCode::FAILURE;
    }
    let runner = PresetRunner::new(config, logger);
    match runner.run(args[0]) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
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
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn cmd_cleanup(args: &[&str], config: &Config) -> ExitCode {
    let dry_run = args.contains(&"--dry-run");
    let all = args.contains(&"--all");

    let mut options = CleanupOptions::new().dry_run(dry_run).delete_all(all);

    if let Some(idx) = args.iter().position(|&a| a == "--older-than") {
        if let Some(days_str) = args.get(idx + 1) {
            if let Ok(days) = days_str.trim_end_matches('d').parse::<u32>() {
                options = options.max_age_days(days);
            }
        }
    }

    if let Some(idx) = args.iter().position(|&a| a == "--max-size") {
        if let Some(size_str) = args.get(idx + 1) {
            options = options.max_total_size(size_str);
        }
    }

    if let Some(idx) = args.iter().position(|&a| a == "--app") {
        if let Some(app) = args.get(idx + 1) {
            options = options.app_filter((*app).to_string());
        }
    }

    let base_dir = expand_path(&config.file.base_dir);
    match cleanup(&base_dir, &options) {
        Ok(result) => {
            let formatter = OutputFormatter::new().colors(config.terminal.colors);
            println!("{}", formatter.format_cleanup(&result, dry_run));
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {e}");
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
    --older-than <days>                Delete files older than N days
    --max-size <size>                  Keep total size under limit (e.g., 500M, 1G)
    --app <name>                       Filter by app name
    --all                              Delete all files
    --dry-run                          Show what would be deleted
  hyprlog help                         Show this help
  hyprlog version                      Show version

Levels: trace, debug, info, warn, error

Examples:
  hyprlog info INIT \"Application started\"
  hyprlog log error NET \"Connection failed\"
  echo '{{\"level\":\"info\",\"scope\":\"TEST\",\"msg\":\"hello\"}}' | hyprlog json"
    );
}

fn build_logger(config: &Config, app_override: Option<&str>) -> Logger {
    let app_name = app_override.unwrap_or(&config.general.app_name);
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
