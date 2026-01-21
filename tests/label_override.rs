//! Tests for `label_override` functionality.

use hyprlog::output::LogRecord;
use hyprlog::{FormatValues, Level, TagConfig};

#[test]
fn format_tag_without_override() {
    let record = LogRecord {
        level: Level::Info,
        scope: "TEST".to_string(),
        message: "test".to_string(),
        values: FormatValues::new(),
        label_override: None,
        app_name: None,
        raw: false,
    };

    let tag_config = TagConfig::default();
    let tag = record.format_tag(&tag_config);

    assert!(tag.contains("INFO"));
}

#[test]
fn format_tag_with_override() {
    let record = LogRecord {
        level: Level::Info,
        scope: "TEST".to_string(),
        message: "test".to_string(),
        values: FormatValues::new(),
        label_override: Some("SUCCESS".to_string()),
        app_name: None,
        raw: false,
    };

    let tag_config = TagConfig::default();
    let tag = record.format_tag(&tag_config);

    assert!(tag.contains("SUCCESS"));
    assert!(!tag.contains("INFO"));
}

#[test]
fn format_with_label_applies_transform() {
    let tag_config = TagConfig::default(); // uppercase by default
    let tag = tag_config.format_with_label(Level::Info, "success");

    assert!(tag.contains("SUCCESS"));
}

#[test]
fn format_with_label_applies_padding() {
    let tag_config = TagConfig::default().min_width(10);
    let tag = tag_config.format_with_label(Level::Info, "OK");

    // Should be padded to 10 chars
    let inner = tag.trim_start_matches('[').trim_end_matches(']');
    assert_eq!(inner.len(), 10);
}
