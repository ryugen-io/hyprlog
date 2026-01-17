//! Log command implementation.

use crate::util::parse_level;
use hl_core::{Logger, internal};
use std::process::ExitCode;

/// Handles `hyprlog log <level> <scope> <msg>`.
#[must_use]
pub fn cmd_log(args: &[&str], logger: &Logger) -> ExitCode {
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

/// Handles `hyprlog <level> <scope> <msg>` shorthand.
#[must_use]
pub fn cmd_log_shorthand(args: &[&str], logger: &Logger) -> ExitCode {
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
