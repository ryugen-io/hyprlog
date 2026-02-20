//! Tests for the Hyprland event formatter.

#![cfg(feature = "hyprland")]

use hyprlog::HyprlandEvent;
use hyprlog::hyprland::formatter::EventFormatter;

#[test]
fn format_known_event_shows_human_name_and_technical() {
    let fmt = EventFormatter::new();
    let event = HyprlandEvent::parse("workspace>>3").unwrap();
    assert_eq!(fmt.format(&event), "workspace changed (workspace): name=3");
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

#[test]
fn format_openwindow_key_value() {
    let mut fmt = EventFormatter::new();
    let event = HyprlandEvent::parse("openwindow>>80a6f50,2,kitty,Kitty Terminal").unwrap();
    fmt.observe(&event);
    assert_eq!(
        fmt.format(&event),
        r#"window opened (openwindow): app=kitty title="Kitty Terminal" ws=2"#
    );
}

#[test]
fn format_closewindow_with_cache() {
    let mut fmt = EventFormatter::new();
    let open = HyprlandEvent::parse("openwindow>>80a6f50,2,rio,rio").unwrap();
    fmt.observe(&open);
    let close = HyprlandEvent::parse("closewindow>>80a6f50").unwrap();
    // format BEFORE observe so cache entry still exists
    let msg = fmt.format(&close);
    assert_eq!(msg, "window closed (closewindow): app=rio");
    fmt.observe(&close); // now remove from cache
}

#[test]
fn format_windowtitlev2() {
    let fmt = EventFormatter::new();
    let event = HyprlandEvent::parse("windowtitlev2>>80a6f50,Yazi: ~/").unwrap();
    assert_eq!(
        fmt.format(&event),
        r#"title changed (windowtitlev2): title="Yazi: ~/""#
    );
}

#[test]
fn format_focusedmonv2() {
    let fmt = EventFormatter::new();
    let event = HyprlandEvent::parse("focusedmonv2>>DP-2,2").unwrap();
    assert_eq!(
        fmt.format(&event),
        "monitor focus (focusedmonv2): monitor=DP-2 id=2"
    );
}

#[test]
fn format_movewindowv2() {
    let fmt = EventFormatter::new();
    let event = HyprlandEvent::parse("movewindowv2>>80a6f50,2,2").unwrap();
    assert_eq!(
        fmt.format(&event),
        "window moved (movewindowv2): ws=2"
    );
}

#[test]
fn format_movewindow() {
    let fmt = EventFormatter::new();
    let event = HyprlandEvent::parse("movewindow>>80a6f50,2").unwrap();
    assert_eq!(
        fmt.format(&event),
        "window moved (movewindow): ws=2"
    );
}

#[test]
fn format_activewindow() {
    let fmt = EventFormatter::new();
    let event = HyprlandEvent::parse("activewindow>>kitty,Kitty Terminal").unwrap();
    assert_eq!(
        fmt.format(&event),
        r#"window focused (activewindow): app=kitty title="Kitty Terminal""#
    );
}

#[test]
fn format_workspace_events() {
    let fmt = EventFormatter::new();

    let event = HyprlandEvent::parse("workspace>>3").unwrap();
    assert_eq!(fmt.format(&event), "workspace changed (workspace): name=3");

    let event = HyprlandEvent::parse("createworkspace>>coding").unwrap();
    assert_eq!(
        fmt.format(&event),
        "workspace created (createworkspace): name=coding"
    );

    let event = HyprlandEvent::parse("destroyworkspace>>coding").unwrap();
    assert_eq!(
        fmt.format(&event),
        "workspace destroyed (destroyworkspace): name=coding"
    );
}

#[test]
fn format_windowtitle_with_cache() {
    let mut fmt = EventFormatter::new();
    let open = HyprlandEvent::parse("openwindow>>80a6f50,2,kitty,Kitty").unwrap();
    fmt.observe(&open);

    let event = HyprlandEvent::parse("windowtitle>>80a6f50").unwrap();
    assert_eq!(fmt.format(&event), "title changed (windowtitle): app=kitty");
}

#[test]
fn format_windowtitle_without_cache() {
    let fmt = EventFormatter::new();
    let event = HyprlandEvent::parse("windowtitle>>80a6f50").unwrap();
    assert_eq!(fmt.format(&event), "title changed (windowtitle): 80a6f50");
}
