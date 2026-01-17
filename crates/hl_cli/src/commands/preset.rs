//! Preset command implementations.

use hl_common::PresetRunner;
use hl_core::{Config, Logger, internal};
use std::process::ExitCode;

/// Handles `hyprlog preset <name>`.
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

/// Handles `hyprlog presets` to list all presets.
#[must_use]
pub fn cmd_presets(config: &Config, logger: &Logger) -> ExitCode {
    let runner = PresetRunner::new(config, logger);
    let list = runner.list();
    if list.is_empty() {
        println!("No presets defined");
    } else {
        println!("Available presets:");
        let mut groups: std::collections::BTreeMap<String, Vec<&str>> =
            std::collections::BTreeMap::new();

        for (name, app_name) in list {
            let key = app_name.unwrap_or("general").to_string();
            groups.entry(key).or_default().push(name);
        }

        for (app, mut presets) in groups {
            println!("[{app}]");
            presets.sort_unstable();
            for preset in presets {
                println!("  {preset}");
            }
        }
    }
    ExitCode::SUCCESS
}
