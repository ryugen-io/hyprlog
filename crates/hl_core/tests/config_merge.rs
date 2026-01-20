use hl_core::Config;
use hl_core::config::PresetConfig;
use std::collections::HashMap;

#[test]
fn merge_preserves_existing_and_adds_missing() {
    let mut base = Config::default();
    base.colors.insert("red".to_string(), "#111111".to_string());
    base.tag
        .labels
        .insert("info".to_string(), "INFO".to_string());
    base.highlight
        .keywords
        .insert("ok".to_string(), "green".to_string());
    base.icons
        .nerdfont
        .insert("info".to_string(), "i".to_string());

    let mut base_presets = HashMap::new();
    base_presets.insert(
        "startup".to_string(),
        PresetConfig {
            level: "info".to_string(),
            as_level: None,
            scope: Some("INIT".to_string()),
            msg: "Start".to_string(),
            app_name: None,
        },
    );
    base.presets = base_presets;

    let mut other = Config::default();
    other
        .colors
        .insert("red".to_string(), "#222222".to_string());
    other
        .colors
        .insert("blue".to_string(), "#0000ff".to_string());
    other
        .tag
        .labels
        .insert("warn".to_string(), "WARN".to_string());
    other
        .highlight
        .keywords
        .insert("fail".to_string(), "red".to_string());
    other
        .icons
        .nerdfont
        .insert("warn".to_string(), "!".to_string());
    other.presets.insert(
        "startup".to_string(),
        PresetConfig {
            level: "info".to_string(),
            as_level: None,
            scope: Some("INIT".to_string()),
            msg: "Override".to_string(),
            app_name: None,
        },
    );
    other.presets.insert(
        "shutdown".to_string(),
        PresetConfig {
            level: "info".to_string(),
            as_level: None,
            scope: Some("INIT".to_string()),
            msg: "Stop".to_string(),
            app_name: None,
        },
    );

    base.merge(other);

    assert_eq!(base.colors["red"], "#111111");
    assert_eq!(base.colors["blue"], "#0000ff");
    assert_eq!(base.tag.labels["info"], "INFO");
    assert_eq!(base.tag.labels["warn"], "WARN");
    assert_eq!(base.highlight.keywords["ok"], "green");
    assert_eq!(base.highlight.keywords["fail"], "red");
    assert_eq!(base.icons.nerdfont["info"], "i");
    assert_eq!(base.icons.nerdfont["warn"], "!");
    assert_eq!(base.presets["startup"].msg, "Start");
    assert_eq!(base.presets["shutdown"].msg, "Stop");
}
