//! Tests for logger functionality.

use hyprlog::config::PresetConfig;
use hyprlog::{Level, Logger};
use std::collections::HashMap;

#[test]
fn builder_default() {
    let logger = Logger::builder().build();
    assert_eq!(logger.min_level(), Level::Info);
    assert_eq!(logger.output_count(), 0);
}

#[test]
fn builder_with_level() {
    let logger = Logger::builder().level(Level::Debug).build();
    assert_eq!(logger.min_level(), Level::Debug);
}

#[test]
fn builder_with_terminal() {
    let logger = Logger::builder().terminal().colors(false).done().build();
    assert_eq!(logger.output_count(), 1);
}

#[test]
fn builder_with_file() {
    let logger = Logger::builder()
        .file()
        .base_dir("/tmp/test")
        .app_name("test")
        .done()
        .build();
    assert_eq!(logger.output_count(), 1);
}

#[test]
fn builder_multiple_outputs() {
    let logger = Logger::builder()
        .level(Level::Trace)
        .terminal()
        .done()
        .file()
        .done()
        .build();
    assert_eq!(logger.output_count(), 2);
}

#[test]
fn log_respects_level() {
    let logger = Logger::builder().level(Level::Warn).build();
    // This should not panic even without outputs
    logger.info("TEST", "should be filtered");
    logger.warn("TEST", "should pass");
}

#[test]
fn preset_not_found() {
    let logger = Logger::builder().build();
    assert!(!logger.preset("nonexistent"));
}

#[test]
fn preset_found() {
    let mut presets = HashMap::new();
    presets.insert(
        "startup".to_string(),
        PresetConfig {
            level: "info".to_string(),
            as_level: None,
            scope: Some("INIT".to_string()),
            msg: "Application started".to_string(),
            app_name: None,
        },
    );
    let logger = Logger::builder().presets(presets).build();
    assert!(logger.has_preset("startup"));
    assert_eq!(logger.preset_count(), 1);
    assert!(logger.preset("startup"));
}
