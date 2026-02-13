//! Tests for Hyprland event parsing.

#![cfg(feature = "hyprland")]

use hyprlog::HyprlandEvent;

#[test]
fn parse_valid_event() {
    let event = HyprlandEvent::parse("openwindow>>80a6f50,2,kitty,Kitty").unwrap();
    assert_eq!(event.name, "openwindow");
    assert_eq!(event.data, "80a6f50,2,kitty,Kitty");
}

#[test]
fn parse_empty_data() {
    let event = HyprlandEvent::parse("configreloaded>>").unwrap();
    assert_eq!(event.name, "configreloaded");
    assert_eq!(event.data, "");
}

#[test]
fn parse_data_with_multiple_arrows() {
    // Data may contain >> characters (e.g., in window titles)
    let event = HyprlandEvent::parse("activewindow>>kitty,foo >> bar").unwrap();
    assert_eq!(event.name, "activewindow");
    assert_eq!(event.data, "kitty,foo >> bar");
}

#[test]
fn parse_no_separator_returns_none() {
    assert!(HyprlandEvent::parse("noseparator").is_none());
}

#[test]
fn parse_empty_name_returns_none() {
    assert!(HyprlandEvent::parse(">>data").is_none());
}

#[test]
fn parse_empty_line_returns_none() {
    assert!(HyprlandEvent::parse("").is_none());
}

#[test]
fn parse_trims_whitespace() {
    let event = HyprlandEvent::parse("  workspace>>3  \n").unwrap();
    assert_eq!(event.name, "workspace");
    assert_eq!(event.data, "3");
}
