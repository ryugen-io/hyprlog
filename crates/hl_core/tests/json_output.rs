//! Tests for JSON database output.

use hl_core::{Level, Logger};
use std::fs;
use tempfile::TempDir;

#[test]
fn builder_with_json() {
    let logger = Logger::builder()
        .json()
        .path("/tmp/test.jsonl")
        .app_name("test")
        .done()
        .build();
    assert_eq!(logger.output_count(), 1);
}

#[test]
fn builder_json_with_other_outputs() {
    let logger = Logger::builder()
        .level(Level::Trace)
        .terminal()
        .done()
        .json()
        .done()
        .build();
    assert_eq!(logger.output_count(), 2);
}

#[test]
fn json_writes_valid_jsonl() {
    let tmp_dir = TempDir::new().unwrap();
    let json_path = tmp_dir.path().join("test.jsonl");

    let logger = Logger::builder()
        .level(Level::Trace)
        .json()
        .path(&json_path)
        .app_name("testapp")
        .done()
        .build();

    logger.info("TEST", "Hello world");
    logger.warn("WARN", "A warning");
    logger.error("ERR", "An error");

    // Read and verify JSONL content
    let content = fs::read_to_string(&json_path).unwrap();
    let lines: Vec<&str> = content.lines().collect();

    assert_eq!(lines.len(), 3);

    // Each line should be valid JSON
    for line in &lines {
        let parsed: serde_json::Value = serde_json::from_str(line).unwrap();

        // Verify required fields exist
        assert!(parsed.get("id").is_some(), "missing id field");
        assert!(parsed.get("ts").is_some(), "missing ts field");
        assert!(parsed.get("level").is_some(), "missing level field");
        assert!(parsed.get("scope").is_some(), "missing scope field");
        assert!(parsed.get("msg").is_some(), "missing msg field");
    }

    // Verify first entry content
    let first: serde_json::Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(first["level"], "info");
    assert_eq!(first["scope"], "TEST");
    assert_eq!(first["msg"], "Hello world");
    assert_eq!(first["app"], "testapp");
}

#[test]
fn json_ulid_is_unique() {
    let tmp_dir = TempDir::new().unwrap();
    let json_path = tmp_dir.path().join("test.jsonl");

    let logger = Logger::builder().json().path(&json_path).done().build();

    // Write multiple entries
    for i in 0..10 {
        logger.info("TEST", &format!("Entry {i}"));
    }

    let content = fs::read_to_string(&json_path).unwrap();
    let ids: Vec<String> = content
        .lines()
        .map(|line| {
            let parsed: serde_json::Value = serde_json::from_str(line).unwrap();
            parsed["id"].as_str().unwrap().to_string()
        })
        .collect();

    // All IDs should be unique
    let mut unique_ids = ids.clone();
    unique_ids.sort();
    unique_ids.dedup();
    assert_eq!(ids.len(), unique_ids.len(), "ULIDs should be unique");
}

#[test]
fn json_strips_style_tags() {
    let tmp_dir = TempDir::new().unwrap();
    let json_path = tmp_dir.path().join("test.jsonl");

    let logger = Logger::builder().json().path(&json_path).done().build();

    logger.info("TEST", "Hello <bold>world</bold> with <red>color</red>");

    let content = fs::read_to_string(&json_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(content.trim()).unwrap();

    assert_eq!(parsed["msg"], "Hello world with color");
}

#[test]
fn json_skips_raw_messages() {
    let tmp_dir = TempDir::new().unwrap();
    let json_path = tmp_dir.path().join("test.jsonl");

    let logger = Logger::builder().json().path(&json_path).done().build();

    logger.info("TEST", "Normal message");
    logger.raw("Raw continuation line");
    logger.info("TEST", "Another normal");

    let content = fs::read_to_string(&json_path).unwrap();

    // Raw messages should be skipped
    assert_eq!(content.lines().count(), 2);
}

#[test]
fn json_creates_parent_dirs() {
    let tmp_dir = TempDir::new().unwrap();
    let json_path = tmp_dir
        .path()
        .join("nested")
        .join("deep")
        .join("test.jsonl");

    let logger = Logger::builder().json().path(&json_path).done().build();

    logger.info("TEST", "Should create dirs");

    assert!(json_path.exists());
}
