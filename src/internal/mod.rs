//! Hyprlog's own diagnostic logger — bootstrapped early so config errors and
//! internal warnings can be reported through the same formatting pipeline.
//!
//! Uses `OnceLock` so the logger is initialized exactly once, even if
//! multiple entry points (CLI, FFI, tests) race to call `init`.

use crate::config::Config;
use crate::fmt::IconSet;
use crate::level::Level;
use crate::logger::Logger;
use std::sync::OnceLock;

static INTERNAL_LOGGER: OnceLock<Logger> = OnceLock::new();

/// Fallback initializer that loads config itself — used when no caller provides one.
///
/// `OnceLock` guarantees only the first call takes effect; later calls are no-ops.
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

/// Preferred initializer — reuses the already-loaded config to avoid double I/O.
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

/// Pre-init calls silently vanish rather than crashing — safe during early startup.
fn log(level: Level, scope: &str, msg: &str) {
    if let Some(logger) = INTERNAL_LOGGER.get() {
        logger.log(level, scope, msg);
    }
}

/// Visible only when internal log level includes Trace — for high-volume instrumentation.
pub fn trace(scope: &str, msg: &str) {
    log(Level::Trace, scope, msg);
}

/// Visible only when internal log level includes Debug — for startup and teardown diagnostics.
pub fn debug(scope: &str, msg: &str) {
    log(Level::Debug, scope, msg);
}

/// Normal operational milestones — config loaded, listener started, etc.
pub fn info(scope: &str, msg: &str) {
    log(Level::Info, scope, msg);
}

/// Non-fatal anomalies — missing optional config, deprecated features, etc.
pub fn warn(scope: &str, msg: &str) {
    log(Level::Warn, scope, msg);
}

/// Unrecoverable failures — I/O errors, invalid state, etc.
pub fn error(scope: &str, msg: &str) {
    log(Level::Error, scope, msg);
}
