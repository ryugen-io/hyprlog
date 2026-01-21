//! Tests for log level functionality.

use hyprlog::Level;

#[test]
fn level_ordering() {
    assert!(Level::Trace < Level::Debug);
    assert!(Level::Debug < Level::Info);
    assert!(Level::Info < Level::Warn);
    assert!(Level::Warn < Level::Error);
}

#[test]
fn level_display() {
    assert_eq!(Level::Trace.to_string(), "trace");
    assert_eq!(Level::Debug.to_string(), "debug");
    assert_eq!(Level::Info.to_string(), "info");
    assert_eq!(Level::Warn.to_string(), "warn");
    assert_eq!(Level::Error.to_string(), "error");
}

#[test]
fn level_from_str() {
    assert_eq!("trace".parse::<Level>().unwrap(), Level::Trace);
    assert_eq!("DEBUG".parse::<Level>().unwrap(), Level::Debug);
    assert_eq!("Info".parse::<Level>().unwrap(), Level::Info);
    assert_eq!("warning".parse::<Level>().unwrap(), Level::Warn);
    assert_eq!("err".parse::<Level>().unwrap(), Level::Error);
}

#[test]
fn level_from_str_invalid() {
    assert!("invalid".parse::<Level>().is_err());
}

#[test]
fn level_default() {
    assert_eq!(Level::default(), Level::Info);
}
