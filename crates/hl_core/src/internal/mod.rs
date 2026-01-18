//! Internal logging for hyprlog itself.
//!
//! hyprlog uses its own Logger for internal messages.

use crate::config::Config;
use crate::fmt::IconSet;
use crate::level::Level;
use crate::logger::Logger;
use std::sync::OnceLock;

static INTERNAL_LOGGER: OnceLock<Logger> = OnceLock::new();

/// Initializes the internal logger from config.
///
/// Should be called once at startup. Subsequent calls are ignored.
pub fn init() {
    let was_init = INTERNAL_LOGGER.get().is_some();
    INTERNAL_LOGGER.get_or_init(|| {
        let config = Config::load().unwrap_or_default();
        build_internal_logger(&config)
    });
    if !was_init {
        debug("INTERNAL", "Initializing internal logger");
        debug("INTERNAL", "Internal logger ready");
    }
}

/// Initializes the internal logger with a specific config.
pub fn init_with_config(config: &Config) {
    let was_init = INTERNAL_LOGGER.get().is_some();
    INTERNAL_LOGGER.get_or_init(|| build_internal_logger(config));
    if !was_init {
        debug("INTERNAL", "Initializing internal logger...");
        debug("INTERNAL", &format!("Log level: {}", config.general.level));
        if config.terminal.enabled {
            debug(
                "INTERNAL",
                &format!(
                    "Terminal: colors={}, icons={}",
                    if config.terminal.colors {
                        "enabled"
                    } else {
                        "disabled"
                    },
                    config.terminal.icons
                ),
            );
        }
        if config.file.enabled {
            debug(
                "INTERNAL",
                &format!("File: base_dir={}", config.file.base_dir),
            );
        }
        debug("INTERNAL", "Internal logger ready");
    }
}

fn build_internal_logger(config: &Config) -> Logger {
    let mut builder = Logger::builder().level(config.parse_level());

    if config.terminal.enabled {
        builder = builder
            .terminal()
            .colors(config.terminal.colors)
            .icons(IconSet::from(config.parse_icon_type()))
            .structure(&config.terminal.structure)
            .highlight_config(config.highlight.clone())
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
            .app_name("hyprlog")
            .done();
    }

    builder.build()
}

/// Logs an internal message.
fn log(level: Level, scope: &str, msg: &str) {
    if let Some(logger) = INTERNAL_LOGGER.get() {
        logger.log(level, scope, msg);
    }
}

/// Log internal trace message.
pub fn trace(scope: &str, msg: &str) {
    log(Level::Trace, scope, msg);
}

/// Log internal debug message.
pub fn debug(scope: &str, msg: &str) {
    log(Level::Debug, scope, msg);
}

/// Log internal info message.
pub fn info(scope: &str, msg: &str) {
    log(Level::Info, scope, msg);
}

/// Log internal warning message.
pub fn warn(scope: &str, msg: &str) {
    log(Level::Warn, scope, msg);
}

/// Log internal error message.
pub fn error(scope: &str, msg: &str) {
    log(Level::Error, scope, msg);
}
