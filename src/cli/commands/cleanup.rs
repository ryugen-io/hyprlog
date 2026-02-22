//! Log directories grow without bound — this command applies retention policies
//! (age, size, count) so users don't have to write cron scripts or remember `find -delete`.

use crate::cleanup::{CleanupOptions, cleanup};
use crate::cli::util::expand_path;
use crate::config::Config;
use crate::internal;
use crate::logger::Logger;
use std::process::ExitCode;

/// Merges config defaults with CLI overrides — CLI flags always win so one-off runs
/// can deviate from the persistent config without editing it.
#[must_use]
pub fn cmd_cleanup(args: &[&str], config: &Config, logger: &Logger) -> ExitCode {
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

    // Config provides baseline retention policy — CLI flags override below
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

    // CLI flags take precedence — one-off runs shouldn't require editing the config file
    if let Some(idx) = args.iter().position(|&a| a == "--older-than")
        && let Some(days_str) = args.get(idx + 1)
        && let Ok(days) = days_str.trim_end_matches('d').parse::<u32>()
    {
        internal::debug("CLEANUP", &format!("CLI override: max_age_days={days}"));
        options = options.max_age_days(days);
    }

    if let Some(idx) = args.iter().position(|&a| a == "--max-size")
        && let Some(size_str) = args.get(idx + 1)
    {
        internal::debug("CLEANUP", &format!("CLI override: max_size={size_str}"));
        options = options.max_total_size(size_str);
    }

    if let Some(idx) = args.iter().position(|&a| a == "--app")
        && let Some(app) = args.get(idx + 1)
    {
        internal::debug("CLEANUP", &format!("CLI override: app={app}"));
        options = options.app_filter((*app).to_string());
    }

    if let Some(idx) = args.iter().position(|&a| a == "--keep-last")
        && let Some(n_str) = args.get(idx + 1)
        && let Ok(n) = n_str.parse::<usize>()
    {
        internal::debug("CLEANUP", &format!("CLI override: keep_last={n}"));
        options = options.keep_last(n);
    }

    if let Some(idx) = args.iter().position(|&a| a == "--before")
        && let Some(date_str) = args.get(idx + 1)
    {
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

    if let Some(idx) = args.iter().position(|&a| a == "--after")
        && let Some(date_str) = args.get(idx + 1)
    {
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

    let base_dir = expand_path(&config.file.base_dir);
    internal::debug("CLEANUP", &format!("Base dir: {}", base_dir.display()));

    match cleanup(&base_dir, &options) {
        Ok(result) => {
            // Individual file failures shouldn't abort the whole cleanup — warn and continue
            for (path, err) in &result.failed {
                internal::warn("CLEANUP", &format!("Failed to process {path}: {err}"));
            }

            result.log(logger, dry_run);
            ExitCode::SUCCESS
        }
        Err(e) => {
            internal::error("CLEANUP", &format!("{e}"));
            ExitCode::FAILURE
        }
    }
}
