//! Tests for SDK-backed Hyprland event conversion.

#![cfg(feature = "hyprland")]

use hypr_sdk::ipc::events::parse_event;
use hyprlog::HyprlandEvent;

#[test]
fn from_sdk_openwindow_preserves_event_shape() {
    let sdk_event = parse_event("openwindow>>80a6f50,2,kitty,Kitty").unwrap();
    let event = HyprlandEvent::from_sdk(&sdk_event);

    assert_eq!(event.name, "openwindow");
    assert_eq!(event.data, "80a6f50,2,kitty,Kitty");
    assert_eq!(event.format_message(), "openwindow: 80a6f50,2,kitty,Kitty");
}

#[test]
fn from_sdk_boolean_events_use_wire_0_1_format() {
    let fullscreen = parse_event("fullscreen>>1").unwrap();
    let screencast = parse_event("screencast>>0,pipewire").unwrap();

    let fullscreen_event = HyprlandEvent::from_sdk(&fullscreen);
    let screencast_event = HyprlandEvent::from_sdk(&screencast);

    assert_eq!(fullscreen_event.name, "fullscreen");
    assert_eq!(fullscreen_event.data, "1");
    assert_eq!(screencast_event.name, "screencast");
    assert_eq!(screencast_event.data, "0,pipewire");
}

#[test]
fn from_sdk_unknown_event_passthrough() {
    let sdk_event = parse_event("madeupevent>>raw,data").unwrap();
    let event = HyprlandEvent::from_sdk(&sdk_event);

    assert_eq!(event.name, "madeupevent");
    assert_eq!(event.data, "raw,data");
}
