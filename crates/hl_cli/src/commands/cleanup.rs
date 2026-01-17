//! Cleanup command implementation.

use crate::util::expand_path;
use hl_common::OutputFormatter;
use hl_core::{CleanupOptions, Config, cleanup, internal};
use std::process::ExitCode;

/// Handles `hyprlog cleanup [options]`.
#[must_use]
pub fn cmd_cleanup(args: &[&str], config: &Config) -> ExitCode {
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
