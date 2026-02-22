//! Thin CLI wrappers around `PresetRunner` — the actual logic lives in `cli::preset`
//! so the REPL can reuse it without depending on `ExitCode` or CLI arg parsing.

use crate::cli::preset::PresetRunner;
use crate::config::Config;
use crate::internal;
use crate::logger::Logger;
use std::process::ExitCode;

/// Delegates to `PresetRunner` — the CLI layer only handles arg validation and exit codes.
#[must_use]
pub fn cmd_preset(args: &[&str], config: &Config, logger: &Logger) -> ExitCode {
    if args.is_empty() {
        internal::warn("CLI", "Usage: hyprlog preset <name>");
        return ExitCode::FAILURE;
    }
    let runner = PresetRunner::new(config, logger);
    match runner.run(args[0]) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            internal::error("PRESET", &format!("{e}"));
            ExitCode::FAILURE
        }
    }
}

/// Groups presets by app name so multi-app configs are scannable — flat lists become unreadable past ~10 presets.
#[must_use]
pub fn cmd_presets(config: &Config, logger: &Logger) -> ExitCode {
    let runner = PresetRunner::new(config, logger);
    let list = runner.list();
    if list.is_empty() {
        logger.print("PRESETS", "No presets defined");
    } else {
        logger.print("PRESETS", "Available presets:");
        let mut groups: std::collections::BTreeMap<String, Vec<&str>> =
            std::collections::BTreeMap::new();

        for (name, app_name) in list {
            let key = app_name.unwrap_or("general").to_string();
            groups.entry(key).or_default().push(name);
        }

        for (app, mut presets) in groups {
            logger.raw(&format!("  [{app}]"));
            presets.sort_unstable();
            for preset in presets {
                logger.raw(&format!("    {preset}"));
            }
        }
    }
    ExitCode::SUCCESS
}
