//! Tests for the Hyprland event formatter.

#![cfg(feature = "hyprland")]

use hyprlog::HyprlandEvent;
use hyprlog::hyprland::formatter::EventFormatter;

#[test]
fn format_known_event_shows_human_name_and_technical() {
    let fmt = EventFormatter::new();
    let event = HyprlandEvent::parse("workspace>>3").unwrap();
    assert_eq!(fmt.format(&event), "workspace changed (workspace): 3");
}

#[test]
fn format_unknown_event_uses_technical_name() {
    let fmt = EventFormatter::new();
    let event = HyprlandEvent::parse("somenewevent>>payload").unwrap();
    assert_eq!(fmt.format(&event), "somenewevent: payload");
}

#[test]
fn format_empty_data_omits_colon() {
    let fmt = EventFormatter::new();
    let event = HyprlandEvent::parse("configreloaded>>").unwrap();
    assert_eq!(fmt.format(&event), "configreloaded");
}

#[test]
fn format_known_event_empty_data_omits_colon() {
    let fmt = EventFormatter::new();
    let event = HyprlandEvent {
        name: "fullscreen".into(),
        data: String::new(),
    };
    assert_eq!(fmt.format(&event), "fullscreen");
}
