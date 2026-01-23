//! Stats command implementation.

use crate::cleanup::stats;
use crate::cli::util::expand_path;
use crate::config::Config;
use crate::internal;
use crate::logger::Logger;
use std::process::ExitCode;

/// Handles `hyprlog stats [--app <name>]`.
#[must_use]
pub fn cmd_stats(args: &[&str], config: &Config, logger: &Logger) -> ExitCode {
    let base_dir = expand_path(&config.file.base_dir);

    // Parse --app filter
    let app_filter = args
        .iter()
        .position(|&a| a == "--app")
        .and_then(|i| args.get(i + 1).copied());

    match stats(&base_dir, app_filter) {
        Ok(s) => {
            s.log(logger);
            ExitCode::SUCCESS
        }
        Err(e) => {
            internal::error("STATS", &format!("{e}"));
            ExitCode::FAILURE
        }
    }
}
