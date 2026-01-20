//! Tests for per-app configuration overrides via `[apps.X]` sections.

use hl_core::Config;
use hl_core::config::{AppConfig, AppFileConfig, AppTerminalConfig};

#[test]
fn for_app_level_override() {
    let mut config = Config::default();
    config.general.level = "info".to_string();

    config.apps.insert(
        "sysrat".to_string(),
        AppConfig {
            level: Some("debug".to_string()),
            ..Default::default()
        },
    );

    let sysrat_config = config.for_app("sysrat");
    assert_eq!(sysrat_config.general.level, "debug");

    // Global config unchanged
    assert_eq!(config.general.level, "info");
}

#[test]
fn for_app_terminal_override() {
    let mut config = Config::default();
    config.terminal.colors = true;
    config.terminal.icons = "nerdfont".to_string();
    config.terminal.structure = "{tag} {scope} {msg}".to_string();

    config.apps.insert(
        "myapp".to_string(),
        AppConfig {
            terminal: Some(AppTerminalConfig {
                enabled: Some(false),
                colors: Some(false),
                icons: Some("ascii".to_string()),
                structure: None, // Keep global default
            }),
            ..Default::default()
        },
    );

    let myapp_config = config.for_app("myapp");
    assert!(!myapp_config.terminal.enabled);
    assert!(!myapp_config.terminal.colors);
    assert_eq!(myapp_config.terminal.icons, "ascii");
    // Structure kept from global
    assert_eq!(myapp_config.terminal.structure, "{tag} {scope} {msg}");
}

#[test]
fn for_app_file_override() {
    let mut config = Config::default();
    config.file.enabled = false;
    config.file.base_dir = "/default/logs".to_string();

    config.apps.insert(
        "server".to_string(),
        AppConfig {
            file: Some(AppFileConfig {
                enabled: Some(true),
                base_dir: Some("/custom/logs".to_string()),
            }),
            ..Default::default()
        },
    );

    let server_config = config.for_app("server");
    assert!(server_config.file.enabled);
    assert_eq!(server_config.file.base_dir, "/custom/logs");
}

#[test]
fn for_app_unknown_app_returns_global() {
    let mut config = Config::default();
    config.general.level = "warn".to_string();
    config.terminal.colors = false;

    // No apps configured
    let unknown_config = config.for_app("unknown_app");

    // Should return global settings unchanged
    assert_eq!(unknown_config.general.level, "warn");
    assert!(!unknown_config.terminal.colors);
}

#[test]
fn for_app_partial_override_keeps_other_global() {
    let mut config = Config::default();
    config.general.level = "info".to_string();
    config.terminal.colors = true;
    config.terminal.icons = "nerdfont".to_string();
    config.file.enabled = true;

    // Only override level, nothing else
    config.apps.insert(
        "minimal".to_string(),
        AppConfig {
            level: Some("error".to_string()),
            ..Default::default()
        },
    );

    let minimal_config = config.for_app("minimal");
    assert_eq!(minimal_config.general.level, "error");
    // Everything else from global
    assert!(minimal_config.terminal.colors);
    assert_eq!(minimal_config.terminal.icons, "nerdfont");
    assert!(minimal_config.file.enabled);
}

#[test]
fn for_app_multiple_apps_independent() {
    let mut config = Config::default();
    config.general.level = "info".to_string();

    config.apps.insert(
        "sysrat".to_string(),
        AppConfig {
            level: Some("debug".to_string()),
            ..Default::default()
        },
    );

    config.apps.insert(
        "other".to_string(),
        AppConfig {
            level: Some("error".to_string()),
            ..Default::default()
        },
    );

    let sysrat_config = config.for_app("sysrat");
    let other_config = config.for_app("other");
    let global_config = config.for_app("nonexistent");

    assert_eq!(sysrat_config.general.level, "debug");
    assert_eq!(other_config.general.level, "error");
    assert_eq!(global_config.general.level, "info");
}
