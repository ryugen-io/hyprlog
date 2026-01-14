//! hyprlog interactive shell.

use hl_common::{OutputFormatter, PresetRunner};
use hl_core::{CleanupOptions, Config, IconSet, Level, Logger, cleanup, stats};
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::{DefaultEditor, Editor};
use std::path::PathBuf;

const PROMPT: &str = "hyprlog> ";

/// Runs the interactive shell.
///
/// # Errors
/// Returns error message if shell cannot be initialized.
pub fn run(config: &Config) -> Result<(), String> {
    let logger = build_logger(config);

    let mut rl: Editor<(), DefaultHistory> =
        DefaultEditor::new().map_err(|e| format!("Error creating editor: {e}"))?;

    let history_path = get_history_path();
    if let Some(path) = &history_path {
        let _ = rl.load_history(path);
    }

    println!("hyprlog shell - type 'help' for commands, 'quit' to exit");

    loop {
        match rl.readline(PROMPT) {
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
                eprintln!("Error: {e}");
                break;
            }
        }
    }

    if let Some(path) = &history_path {
        let _ = rl.save_history(path);
    }

    Ok(())
}

fn handle_command(line: &str, config: &Config, logger: &Logger) -> bool {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return true;
    }

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
            cmd_stats(config);
            true
        }
        "cleanup" => {
            cmd_cleanup(&parts, config);
            true
        }
        _ => {
            eprintln!("Unknown command: {}", parts[0]);
            eprintln!("Type 'help' for available commands");
            true
        }
    }
}

fn cmd_log(parts: &[&str], logger: &Logger) {
    if parts.len() < 4 {
        eprintln!("Usage: log <level> <scope> <message>");
        return;
    }
    let Some(level) = parse_level(parts[1]) else {
        eprintln!("Invalid level: {}", parts[1]);
        return;
    };
    logger.log(level, parts[2], &parts[3..].join(" "));
}

fn cmd_log_shorthand(parts: &[&str], logger: &Logger) {
    if parts.len() < 3 {
        eprintln!("Usage: {} <scope> <message>", parts[0]);
        return;
    }
    let Some(level) = parse_level(parts[0]) else {
        return;
    };
    logger.log(level, parts[1], &parts[2..].join(" "));
}

fn cmd_preset(parts: &[&str], config: &Config, logger: &Logger) {
    if parts.len() < 2 {
        eprintln!("Usage: preset <name>");
        return;
    }
    let runner = PresetRunner::new(config, logger);
    if let Err(e) = runner.run(parts[1]) {
        eprintln!("Error: {e}");
    }
}

fn cmd_presets(config: &Config, logger: &Logger) {
    let runner = PresetRunner::new(config, logger);
    let list = runner.list();
    if list.is_empty() {
        println!("No presets defined");
    } else {
        println!("Available presets:");
        for name in list {
            println!("  {name}");
        }
    }
}

fn cmd_stats(config: &Config) {
    let base_dir = expand_path(&config.file.base_dir);
    match stats(&base_dir, None) {
        Ok(s) => {
            let formatter = OutputFormatter::new().colors(config.terminal.colors);
            println!("{}", formatter.format_stats(&s));
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}

fn cmd_cleanup(parts: &[&str], config: &Config) {
    let dry_run = parts.contains(&"--dry-run");
    let all = parts.contains(&"--all");

    let mut options = CleanupOptions::new().dry_run(dry_run).delete_all(all);

    if let Some(idx) = parts.iter().position(|&p| p == "--older-than") {
        if let Some(days_str) = parts.get(idx + 1) {
            if let Ok(days) = days_str.trim_end_matches('d').parse::<u32>() {
                options = options.max_age_days(days);
            }
        }
    }

    if let Some(idx) = parts.iter().position(|&p| p == "--max-size") {
        if let Some(size_str) = parts.get(idx + 1) {
            options = options.max_total_size(size_str);
        }
    }

    let base_dir = expand_path(&config.file.base_dir);
    match cleanup(&base_dir, &options) {
        Ok(result) => {
            let formatter = OutputFormatter::new().colors(config.terminal.colors);
            println!("{}", formatter.format_cleanup(&result, dry_run));
        }
        Err(e) => eprintln!("Error: {e}"),
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
  log <level> <scope> <message>  Log a message
  <level> <scope> <message>      Shorthand (trace, debug, info, warn, error)
  preset <name>                  Run a preset
  presets                        List available presets
  stats                          Show log statistics
  cleanup [options]              Clean up old logs
    --older-than <days>          Delete files older than N days
    --max-size <size>            Keep total size under limit
    --all                        Delete all files
    --dry-run                    Show what would be deleted
  help, ?                        Show this help
  quit, exit, q                  Exit shell"
    );
}

fn build_logger(config: &Config) -> Logger {
    let mut builder = Logger::builder().level(config.parse_level());

    if config.terminal.enabled {
        builder = builder
            .terminal()
            .colors(config.terminal.colors)
            .icons(IconSet::from(config.parse_icon_type()))
            .structure(&config.terminal.structure)
            .done();
    }

    if config.file.enabled {
        builder = builder
            .file()
            .base_dir(&config.file.base_dir)
            .path_structure(&config.file.path_structure)
            .filename_structure(&config.file.filename_structure)
            .content_structure(&config.file.content_structure)
            .timestamp_format(&config.file.timestamp_format)
            .app_name(&config.general.app_name)
            .done();
    }

    builder.build()
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
