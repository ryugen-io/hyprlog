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

mod commands;
mod util;

use commands::{
    cmd_cleanup, cmd_json, cmd_log, cmd_log_shorthand, cmd_preset, cmd_presets, cmd_stats,
};
use hl_core::{Config, internal};
use std::process::ExitCode;
use util::{build_logger, parse_level, print_help};

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
        // Shorthand: hyprlog <level> <scope> <msg>
        "trace" | "debug" | "info" | "warn" | "error" => cmd_log_shorthand(&args_str, &logger),
        // Shorthand with app: hyprlog <app> <level> <scope> <msg>
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
