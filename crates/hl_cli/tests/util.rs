//! Tests for CLI utility functions.

use hl_cli::util::{expand_path, parse_level};
use hl_core::Level;

#[test]
fn parse_level_valid() {
    assert_eq!(parse_level("trace"), Some(Level::Trace));
    assert_eq!(parse_level("debug"), Some(Level::Debug));
    assert_eq!(parse_level("info"), Some(Level::Info));
    assert_eq!(parse_level("warn"), Some(Level::Warn));
    assert_eq!(parse_level("error"), Some(Level::Error));
}

#[test]
fn parse_level_case_insensitive() {
    assert_eq!(parse_level("TRACE"), Some(Level::Trace));
    assert_eq!(parse_level("Debug"), Some(Level::Debug));
    assert_eq!(parse_level("INFO"), Some(Level::Info));
    assert_eq!(parse_level("Warn"), Some(Level::Warn));
    assert_eq!(parse_level("ERROR"), Some(Level::Error));
}

#[test]
fn parse_level_invalid() {
    assert_eq!(parse_level("invalid"), None);
    assert_eq!(parse_level(""), None);
    assert_eq!(parse_level("warning"), None);
    assert_eq!(parse_level("err"), None);
}

#[test]
fn expand_path_no_tilde() {
    let path = expand_path("/absolute/path");
    assert_eq!(path.to_str().unwrap(), "/absolute/path");
}

#[test]
fn expand_path_relative() {
    let path = expand_path("relative/path");
    assert_eq!(path.to_str().unwrap(), "relative/path");
}

#[test]
fn expand_path_with_tilde() {
    let path = expand_path("~/test");
    // Should expand tilde to home directory
    assert!(!path.to_str().unwrap().starts_with('~'));
    assert!(path.to_str().unwrap().ends_with("/test"));
}
