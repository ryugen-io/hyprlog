//! Users can't make informed cleanup decisions without knowing how much disk
//! space logs consume and how they're distributed across apps.

use crate::cleanup::stats;
use crate::cli::util::expand_path;
use crate::config::Config;
use crate::internal;
use crate::logger::Logger;
use std::process::ExitCode;

/// Optional app filter narrows stats to a single app — useful when one app dominates disk usage.
#[must_use]
pub fn cmd_stats(args: &[&str], config: &Config, logger: &Logger) -> ExitCode {
    let base_dir = expand_path(&config.file.base_dir);

    // Without a filter, stats cover all apps — the filter isolates a single app's numbers
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
