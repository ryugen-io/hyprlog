//! Themes command implementation.

use crate::logger::Logger;
use crate::shell::themes::{ALL_THEMES, Theme};
use std::process::ExitCode;

/// Handles `hyprlog themes [list|preview]`.
#[must_use]
pub fn cmd_themes(args: &[&str], logger: &Logger) -> ExitCode {
    match args.first().copied() {
        Some("list") | None => {
            logger.print("THEMES", "Available themes:");
            for theme in ALL_THEMES {
                let marker = if *theme == Theme::default() {
                    " (default)"
                } else {
                    ""
                };
                logger.raw(&format!("  {}{}", theme.name(), marker));
            }
        }
        Some("preview") => {
            logger.print("THEMES", "Theme previews:");
            for theme in ALL_THEMES {
                let prompt = theme.build_prompt();
                logger.raw(&format!("  {}: {prompt}", theme.name()));
            }
        }
        Some(name) => {
            logger.error("THEMES", &format!("Unknown subcommand: {name}"));
            logger.print("THEMES", "Usage: hyprlog themes [list|preview]");
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}
