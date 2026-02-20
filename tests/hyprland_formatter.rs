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

#[test]
fn observe_openwindow_populates_cache() {
    let mut fmt = EventFormatter::new();
    let open = HyprlandEvent::parse("openwindow>>80a6f50,2,kitty,Kitty").unwrap();
    fmt.observe(&open);

    let urgent = HyprlandEvent::parse("urgent>>80a6f50").unwrap();
    let msg = fmt.format(&urgent);
    assert_eq!(msg, "focus requested (urgent): app=kitty");
}

#[test]
fn observe_closewindow_removes_cache_entry() {
    let mut fmt = EventFormatter::new();
    let open = HyprlandEvent::parse("openwindow>>80a6f50,2,kitty,Kitty").unwrap();
    fmt.observe(&open);

    let close = HyprlandEvent::parse("closewindow>>80a6f50").unwrap();
    fmt.observe(&close);

    let urgent = HyprlandEvent::parse("urgent>>80a6f50").unwrap();
    let msg = fmt.format(&urgent);
    // No cached app, falls back to raw data
    assert_eq!(msg, "focus requested (urgent): 80a6f50");
}

#[test]
fn urgent_without_cache_shows_raw_address() {
    let fmt = EventFormatter::new();
    let event = HyprlandEvent::parse("urgent>>80a6f50").unwrap();
    assert_eq!(fmt.format(&event), "focus requested (urgent): 80a6f50");
}
