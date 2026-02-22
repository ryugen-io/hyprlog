//! The most common CLI operation — every shell script and automation tool
//! needs a way to emit a single structured log line without starting the REPL.

use crate::cli::util::parse_level;
use crate::internal;
use crate::logger::Logger;
use std::process::ExitCode;

/// Explicit form — the app name is required, no ambiguity about which field is which.
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

/// Shorthand form — typing `hyprlog info SCOPE msg` is faster than `hyprlog log myapp info SCOPE msg`.
/// Ambiguity is resolved by checking if the first arg parses as a valid level.
#[must_use]
pub fn cmd_log_shorthand(args: &[&str], logger: &Logger) -> ExitCode {
    if args.len() < 3 {
        internal::warn("CLI", "Usage: hyprlog [<app>] <level> <scope> <message>");
        return ExitCode::FAILURE;
    }

    // If first arg is a valid level, there's no app name — otherwise it must be the app
    if let Some(level) = parse_level(args[0]) {
        // No app name given — logger defaults to the binary name detected at startup
        logger.log(level, args[1], &args[2..].join(" "));
    } else {
        // First arg isn't a level, so it must be an app name — need one more arg for the actual level
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
