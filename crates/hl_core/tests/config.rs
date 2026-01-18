//! Tests for configuration functionality.

use hl_core::Transform;
use hl_core::{Config, Level};

#[test]
fn default_config() {
    let config = Config::default();
    assert_eq!(config.general.level, "info");
    assert!(config.terminal.enabled);
    assert!(!config.file.enabled);
}

#[test]
fn parse_level() {
    let mut config = Config::default();
    config.general.level = "debug".to_string();
    assert_eq!(config.parse_level(), Level::Debug);
}

#[test]
fn parse_toml() {
    let toml = r#"
[general]
level = "debug"
app_name = "testapp"

[terminal]
enabled = true
colors = false
icons = "ascii"

[tag]
prefix = "<"
suffix = ">"
transform = "lowercase"
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(config.general.level, "debug");
    assert_eq!(config.general.app_name, Some("testapp".to_string()));
    assert!(!config.terminal.colors);
    assert_eq!(config.tag.prefix, "<");
    assert_eq!(config.parse_transform(), Transform::Lowercase);
}

#[test]
fn parse_colors() {
    let toml = r##"
[colors]
red = "#ff0000"
green = "#00ff00"
"##;
    let config: Config = toml::from_str(toml).unwrap();
    let red = config.get_color("red").unwrap();
    assert_eq!(red.r, 255);
    assert_eq!(red.g, 0);
}

#[test]
fn parse_presets() {
    let toml = r#"
[presets.startup]
level = "info"
scope = "INIT"
msg = "Application started"

[presets.shutdown]
level = "info"
scope = "INIT"
msg = "Application stopped"
"#;
    let config: Config = toml::from_str(toml).unwrap();
    assert_eq!(config.presets.len(), 2);
    assert_eq!(config.presets["startup"].scope, Some("INIT".to_string()));
}
