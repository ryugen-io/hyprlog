//! Tests for Hyprland event-to-level mapping.

#![cfg(feature = "hyprland")]

use hyprlog::Level;
use hyprlog::hyprland::level_map::{default_level_map, resolve_level};
use std::collections::HashMap;

#[test]
fn default_map_contains_openwindow() {
    let map = default_level_map();
    assert_eq!(map.get("openwindow"), Some(&Level::Info));
}

#[test]
fn default_map_contains_activewindow() {
    let map = default_level_map();
    assert_eq!(map.get("activewindow"), Some(&Level::Debug));
}

#[test]
fn default_map_contains_urgent() {
    let map = default_level_map();
    assert_eq!(map.get("urgent"), Some(&Level::Warn));
}

#[test]
fn resolve_uses_default_map() {
    let overrides = HashMap::new();
    assert_eq!(resolve_level("openwindow", &overrides), Level::Info);
    assert_eq!(resolve_level("activewindow", &overrides), Level::Debug);
    assert_eq!(resolve_level("urgent", &overrides), Level::Warn);
}

#[test]
fn resolve_unknown_event_returns_info() {
    let overrides = HashMap::new();
    assert_eq!(resolve_level("unknownevent", &overrides), Level::Info);
}

#[test]
fn resolve_user_override_takes_priority() {
    let mut overrides = HashMap::new();
    overrides.insert("openwindow".to_string(), "error".to_string());
    assert_eq!(resolve_level("openwindow", &overrides), Level::Error);
}

#[test]
fn resolve_invalid_user_override_falls_through() {
    let mut overrides = HashMap::new();
    overrides.insert("openwindow".to_string(), "notavalidlevel".to_string());
    // Should fall through to the default map
    assert_eq!(resolve_level("openwindow", &overrides), Level::Info);
}

#[test]
fn resolve_user_override_for_unknown_event() {
    let mut overrides = HashMap::new();
    overrides.insert("customevent".to_string(), "warn".to_string());
    assert_eq!(resolve_level("customevent", &overrides), Level::Warn);
}
