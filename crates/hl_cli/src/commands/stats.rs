//! Stats command implementation.

use crate::util::expand_path;
use hl_common::OutputFormatter;
use hl_core::{Config, internal, stats};
use std::process::ExitCode;

/// Handles `hyprlog stats [--app <name>]`.
#[must_use]
pub fn cmd_stats(args: &[&str], config: &Config) -> ExitCode {
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
