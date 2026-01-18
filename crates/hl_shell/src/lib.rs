//! hyprlog interactive shell.

use hl_common::PresetRunner;
use hl_core::{CleanupOptions, Config, Level, Logger, cleanup, internal, stats};
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::{DefaultEditor, Editor};
use std::fmt::Write;
use std::path::PathBuf;

/// Dracula gradient colors for the prompt.
const GRADIENT: &[(u8, u8, u8)] = &[
    (255, 85, 85),   // #ff5555 red
    (255, 184, 108), // #ffb86c orange
    (241, 250, 140), // #f1fa8c yellow
    (80, 250, 123),  // #50fa7b green
    (139, 233, 253), // #8be9fd cyan
    (189, 147, 249), // #bd93f9 purple
    (255, 121, 198), // #ff79c6 pink
    (255, 85, 85),   // #ff5555 red (for >)
];

/// Builds the gradient-colored prompt.
fn build_prompt() -> String {
    let chars = ['h', 'y', 'p', 'r', 'l', 'o', 'g', '>'];
    let mut prompt = String::new();
    for (i, c) in chars.iter().enumerate() {
        let (r, g, b) = GRADIENT[i];
        let _ = write!(prompt, "\x1b[38;2;{r};{g};{b}m{c}");
    }
    prompt.push_str("\x1b[0m "); // reset + space
    prompt
}

/// Runs the interactive shell.
///
/// # Errors
/// Returns error message if shell cannot be initialized.
pub fn run(config: &Config) -> Result<(), String> {
    let logger = build_logger(config);
    let prompt = build_prompt();

    let mut rl: Editor<(), DefaultHistory> =
        DefaultEditor::new().map_err(|e| format!("Error creating editor: {e}"))?;

    let history_path = get_history_path();
    if let Some(path) = &history_path {
        if rl.load_history(path).is_err() {
            internal::warn("SHELL", "Could not load history");
        }
    }

    internal::info("SHELL", "Starting interactive shell");
    logger.info(
        "SHELL",
        "hyprlog shell - type 'help' for commands, 'quit' to exit",
    );

    loop {
        match rl.readline(&prompt) {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                let _ = rl.add_history_entry(line);

                if !handle_command(line, config, &logger) {
                    break;
                }
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                break;
            }
            Err(e) => {
                internal::error("SHELL", &format!("Readline error: {e}"));
                break;
            }
        }
    }

    if let Some(path) = &history_path {
        if rl.save_history(path).is_err() {
            internal::warn("SHELL", "Could not save history");
        }
    }

    internal::info("SHELL", "Shell exited");
    Ok(())
}

fn handle_command(line: &str, config: &Config, logger: &Logger) -> bool {
    internal::trace("SHELL", &format!("Parsing: {line}"));
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return true;
    }

    internal::trace("SHELL", &format!("Executing: {}", parts[0]));
    match parts[0] {
        "quit" | "exit" | "q" => false,
        "help" | "?" => {
            print_help();
            true
        }
        "log" => {
            cmd_log(&parts, logger);
            true
        }
        "trace" | "debug" | "info" | "warn" | "error" => {
            cmd_log_shorthand(&parts, logger);
            true
        }
        "preset" => {
            cmd_preset(&parts, config, logger);
            true
        }
        "presets" => {
            cmd_presets(config, logger);
            true
        }
        "stats" => {
            cmd_stats(config, logger);
            true
        }
        "cleanup" => {
            cmd_cleanup(&parts, config, logger);
            true
        }
        _ => {
            internal::error("SHELL", &format!("Unknown command: {}", parts[0]));
            internal::info("SHELL", "Type 'help' for available commands");
            true
        }
    }
}

fn cmd_log(parts: &[&str], logger: &Logger) {
    // parts[0] = "log", parts[1] = app, parts[2] = level, parts[3] = scope, parts[4..] = message
    if parts.len() < 5 {
        internal::warn("SHELL", "Usage: log <app> <level> <scope> <message>");
        return;
    }
    let app = parts[1];
    let Some(level) = parse_level(parts[2]) else {
        internal::error("SHELL", &format!("Invalid level: {}", parts[2]));
        return;
    };
    logger.log_full(level, parts[3], &parts[4..].join(" "), Some(app));
}

fn cmd_log_shorthand(parts: &[&str], logger: &Logger) {
    // If parts[1] is a valid level, then parts[0] is app name
    // Otherwise parts[0] is level (and app defaults to "hyprlog")
    if parts.len() < 3 {
        internal::warn("SHELL", &format!("Usage: {} <scope> <message>", parts[0]));
        return;
    }

    // Check if second arg is a level (meaning first arg is app name)
    if parts.len() >= 4 && parse_level(parts[1]).is_some() {
        // parts[0] = app, parts[1] = level, parts[2] = scope, parts[3..] = message
        let app = parts[0];
        let level = parse_level(parts[1]).unwrap();
        logger.log_full(level, parts[2], &parts[3..].join(" "), Some(app));
    } else {
        // parts[0] = level, parts[1] = scope, parts[2..] = message
        let Some(level) = parse_level(parts[0]) else {
            internal::error("SHELL", &format!("Invalid level: {}", parts[0]));
            return;
        };
        logger.log(level, parts[1], &parts[2..].join(" "));
    }
}

fn cmd_preset(parts: &[&str], config: &Config, logger: &Logger) {
    if parts.len() < 2 {
        internal::warn("SHELL", "Usage: preset <name>");
        return;
    }
    let runner = PresetRunner::new(config, logger);
    if let Err(e) = runner.run(parts[1]) {
        internal::error("PRESET", &format!("{e}"));
    }
}

fn cmd_presets(config: &Config, logger: &Logger) {
    let runner = PresetRunner::new(config, logger);
    let list = runner.list();
    if list.is_empty() {
        logger.info("PRESETS", "No presets defined");
    } else {
        logger.info("PRESETS", "Available presets:");
        let mut groups: std::collections::BTreeMap<String, Vec<&str>> =
            std::collections::BTreeMap::new();

        for (name, app_name) in list {
            let key = app_name.unwrap_or("general").to_string();
            groups.entry(key).or_default().push(name);
        }

        for (app, mut presets) in groups {
            logger.info("PRESETS", &format!("[{app}]"));
            presets.sort_unstable();
            for preset in presets {
                logger.info("PRESETS", &format!("  {preset}"));
            }
        }
    }
}

fn cmd_stats(config: &Config, logger: &Logger) {
    let base_dir = expand_path(&config.file.base_dir);
    match stats(&base_dir, None) {
        Ok(s) => s.log(logger),
        Err(e) => internal::error("STATS", &format!("{e}")),
    }
}

fn cmd_cleanup(parts: &[&str], config: &Config, logger: &Logger) {
    let dry_run = parts.contains(&"--dry-run");
    let all = parts.contains(&"--all");

    internal::debug("CLEANUP", &format!("dry_run={dry_run}, all={all}"));

    let mut options = CleanupOptions::new().dry_run(dry_run).delete_all(all);

    if let Some(idx) = parts.iter().position(|&p| p == "--older-than") {
        if let Some(days_str) = parts.get(idx + 1) {
            if let Ok(days) = days_str.trim_end_matches('d').parse::<u32>() {
                internal::debug("CLEANUP", &format!("max_age_days={days}"));
                options = options.max_age_days(days);
            }
        }
    }

    if let Some(idx) = parts.iter().position(|&p| p == "--max-size") {
        if let Some(size_str) = parts.get(idx + 1) {
            internal::debug("CLEANUP", &format!("max_size={size_str}"));
            options = options.max_total_size(size_str);
        }
    }

    let base_dir = expand_path(&config.file.base_dir);
    internal::debug("CLEANUP", &format!("Base dir: {}", base_dir.display()));

    match cleanup(&base_dir, &options) {
        Ok(result) => {
            for (path, err) in &result.failed {
                internal::warn("CLEANUP", &format!("Failed to process {path}: {err}"));
            }
            result.log(logger, dry_run);
        }
        Err(e) => internal::error("CLEANUP", &format!("{e}")),
    }
}

fn parse_level(s: &str) -> Option<Level> {
    match s.to_lowercase().as_str() {
        "trace" => Some(Level::Trace),
        "debug" => Some(Level::Debug),
        "info" => Some(Level::Info),
        "warn" => Some(Level::Warn),
        "error" => Some(Level::Error),
        _ => None,
    }
}

fn print_help() {
    println!(
        "Commands:
  log <app> <level> <scope> <message>   Log a message for specific app
  [<app>] <level> <scope> <message>     Shorthand (app defaults to 'hyprlog')
  preset <name>                         Run a preset
  presets                               List available presets
  stats                                 Show log statistics
  cleanup [options]                     Clean up old logs
    --older-than <days>                 Delete files older than N days
    --max-size <size>                   Keep total size under limit
    --all                               Delete all files
    --dry-run                           Show what would be deleted
  help, ?                               Show this help
  quit, exit, q                         Exit shell

Levels: trace, debug, info, warn, error"
    );
}

fn build_logger(config: &Config) -> Logger {
    Logger::from_config_with(config, "hyprlog")
}

fn expand_path(path: &str) -> PathBuf {
    if path.starts_with('~') {
        if let Some(user_dirs) = directories::UserDirs::new() {
            return PathBuf::from(path.replacen(
                '~',
                user_dirs.home_dir().to_str().unwrap_or(""),
                1,
            ));
        }
    }
    PathBuf::from(path)
}

fn get_history_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "hyprlog")
        .map(|dirs| dirs.data_dir().join("shell_history"))
}
