//! Hyprland users expect a single `hyprlog` binary that "just works" —
//! bare invocation drops into the REPL (matching Hyprland's shell-first convention),
//! while subcommands support scriptable one-shot operations.
//!
//! Usage:
//!   hyprlog                              Enter interactive shell
//!   hyprlog log <app> <level> <scope> <msg>    Log a message
//!   hyprlog <level> <scope> <msg>        Shorthand logging
//!   hyprlog json [<json>]                Log from JSON (or stdin)
//!   hyprlog preset <name>                Run a preset
//!   hyprlog presets                      List presets
//!   hyprlog stats                        Show statistics
//!   hyprlog cleanup [options]            Clean up logs
//!   hyprlog help                         Show help

#[cfg(feature = "hyprland")]
use hyprlog::cli::cmd_watch;
use hyprlog::cli::{build_logger, parse_level, print_help};
use hyprlog::cli::{
    cmd_cleanup, cmd_json, cmd_log, cmd_log_shorthand, cmd_preset, cmd_presets, cmd_stats,
    cmd_themes,
};
use hyprlog::config::Config;
use hyprlog::internal;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Config drives output paths, log level, and formatting — must load before any logger is created
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {e}");
            return ExitCode::FAILURE;
        }
    };

    // Internal logger must be ready before any command runs so diagnostic messages are captured
    internal::init_with_config(&config);

    // Default to the interactive REPL when invoked without arguments (matches Hyprland CLI convention)
    if args.is_empty() {
        return match hyprlog::shell::run(&config) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                internal::error("SHELL", &format!("Shell error: {e}"));
                ExitCode::FAILURE
            }
        };
    }

    // All subcommands share a single logger instance built from config so output is consistent
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
        "stats" => cmd_stats(&args_str[1..], &config, &logger),
        "cleanup" => cmd_cleanup(&args_str[1..], &config, &logger),
        "themes" => cmd_themes(&args_str[1..], &logger),
        #[cfg(feature = "hyprland")]
        "watch" => cmd_watch(&args_str[1..], &config, &logger),
        // Allow level-first invocation for quick one-liners without typing "log" subcommand
        "trace" | "debug" | "info" | "warn" | "error" => cmd_log_shorthand(&args_str, &logger),
        // Detect app-prefixed shorthand (e.g., `hyprlog myapp info SCOPE msg`) by checking if second arg is a valid level
        _ if args_str.len() >= 2 && parse_level(args_str[1]).is_some() => {
            cmd_log_shorthand(&args_str, &logger)
        }
        _ => {
            internal::error("CLI", &format!("Unknown command: {}", args_str[0]));
            internal::info("CLI", "Run 'hyprlog help' for usage");
            ExitCode::FAILURE
        }
    }
}
