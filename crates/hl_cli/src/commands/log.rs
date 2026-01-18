//! Log command implementation.

use crate::util::parse_level;
use hl_core::{Logger, internal};
use std::process::ExitCode;

/// Handles `hyprlog log <app> <level> <scope> <msg>`.
#[must_use]
pub fn cmd_log(args: &[&str], logger: &Logger) -> ExitCode {
    if args.len() < 4 {
        internal::warn("CLI", "Usage: hyprlog log <app> <level> <scope> <message>");
        return ExitCode::FAILURE;
    }
    let app = args[0];
    let Some(level) = parse_level(args[1]) else {
        internal::error("CLI", &format!("Invalid level: {}", args[1]));
        return ExitCode::FAILURE;
    };
    logger.log_full(level, args[2], &args[3..].join(" "), Some(app));
    ExitCode::SUCCESS
}

/// Handles `hyprlog [<app>] <level> <scope> <msg>` shorthand.
/// If first arg is not a valid level, it's treated as app name.
#[must_use]
pub fn cmd_log_shorthand(args: &[&str], logger: &Logger) -> ExitCode {
    if args.len() < 3 {
        internal::warn("CLI", "Usage: hyprlog [<app>] <level> <scope> <message>");
        return ExitCode::FAILURE;
    }

    // Check if first arg is a level or an app name
    if let Some(level) = parse_level(args[0]) {
        // First arg is level -> no app specified, use default
        logger.log(level, args[1], &args[2..].join(" "));
    } else {
        // First arg is app name
        if args.len() < 4 {
            internal::warn("CLI", "Usage: hyprlog <app> <level> <scope> <message>");
            return ExitCode::FAILURE;
        }
        let app = args[0];
        let Some(level) = parse_level(args[1]) else {
            internal::error("CLI", &format!("Invalid level: {}", args[1]));
            return ExitCode::FAILURE;
        };
        logger.log_full(level, args[2], &args[3..].join(" "), Some(app));
    }
    ExitCode::SUCCESS
}
