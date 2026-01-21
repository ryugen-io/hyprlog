//! Tests for cleanup module.

use hyprlog::{CleanupOptions, cleanup, format_size, parse_size, stats};
use std::fs;
use tempfile::tempdir;

#[test]
fn parse_size_bytes() {
    assert_eq!(parse_size("100"), Some(100));
    assert_eq!(parse_size("1K"), Some(1024));
    assert_eq!(parse_size("1KB"), Some(1024));
    assert_eq!(parse_size("1M"), Some(1024 * 1024));
    assert_eq!(parse_size("1MB"), Some(1024 * 1024));
    assert_eq!(parse_size("1G"), Some(1024 * 1024 * 1024));
    assert_eq!(parse_size("500M"), Some(500 * 1024 * 1024));
}

#[test]
fn format_size_bytes() {
    assert_eq!(format_size(100), "100 B");
    assert_eq!(format_size(1024), "1.00 KB");
    assert_eq!(format_size(1024 * 1024), "1.00 MB");
    assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
}

#[test]
fn cleanup_empty_base() {
    let dir = tempdir().unwrap();
    let result = cleanup(dir.path(), &CleanupOptions::default()).unwrap();
    assert_eq!(result.count(), 0);
}

#[test]
fn stats_empty() {
    let dir = tempdir().unwrap();
    let stats = stats(dir.path(), None).unwrap();
    assert_eq!(stats.total_files, 0);
    assert_eq!(stats.total_size, 0);
}

#[test]
fn stats_with_files() {
    let dir = tempdir().unwrap();

    // Create test log files
    let log1 = dir.path().join("test1.log");
    let log2 = dir.path().join("test2.log");
    fs::write(&log1, "test content 1").unwrap();
    fs::write(&log2, "test content 2 longer").unwrap();

    let stats = stats(dir.path(), None).unwrap();
    assert_eq!(stats.total_files, 2);
    assert!(stats.total_size > 0);
}

#[test]
fn cleanup_dry_run() {
    let dir = tempdir().unwrap();

    let log = dir.path().join("test.log");
    fs::write(&log, "test content").unwrap();

    let options = CleanupOptions::new().delete_all(true).dry_run(true);
    let result = cleanup(dir.path(), &options).unwrap();

    assert_eq!(result.would_delete.len(), 1);
    assert!(result.would_free > 0);
    assert!(result.deleted.is_empty());

    // File should still exist
    assert!(log.exists());
}

#[test]
fn cleanup_delete_all() {
    let dir = tempdir().unwrap();

    let log = dir.path().join("test.log");
    fs::write(&log, "test content").unwrap();

    let options = CleanupOptions::new().delete_all(true);
    let result = cleanup(dir.path(), &options).unwrap();

    assert_eq!(result.deleted.len(), 1);
    assert!(result.freed > 0);

    // File should be deleted
    assert!(!log.exists());
}

#[test]
fn cleanup_options_builder() {
    let options = CleanupOptions::new()
        .max_age_days(30)
        .max_total_size("500M")
        .app_filter("myapp")
        .dry_run(true);

    assert_eq!(options.max_age_days, Some(30));
    assert_eq!(options.max_total_size, Some(500 * 1024 * 1024));
    assert_eq!(options.app_filter, Some("myapp".to_string()));
    assert!(options.dry_run);
}
