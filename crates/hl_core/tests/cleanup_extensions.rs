//! Tests for cleanup feature extensions.

use hl_core::{CleanupOptions, cleanup};
use std::fs;
use tempfile::tempdir;

#[test]
fn cleanup_keep_last_protects_newest_files() {
    let dir = tempdir().unwrap();

    // Create 5 log files
    for i in 1..=5 {
        let log = dir.path().join(format!("test{i}.log"));
        fs::write(&log, format!("content {i}")).unwrap();
    }

    // Delete all but keep last 3
    let options = CleanupOptions::new().delete_all(true).keep_last(3);
    let result = cleanup(dir.path(), &options).unwrap();

    // Should delete 2 files
    assert_eq!(result.deleted.len(), 2);

    // 3 files should remain
    let remaining: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "log"))
        .collect();
    assert_eq!(remaining.len(), 3);
}

#[test]
fn cleanup_keep_last_dry_run() {
    let dir = tempdir().unwrap();

    for i in 1..=5 {
        let log = dir.path().join(format!("test{i}.log"));
        fs::write(&log, format!("content {i}")).unwrap();
    }

    let options = CleanupOptions::new()
        .delete_all(true)
        .keep_last(2)
        .dry_run(true);
    let result = cleanup(dir.path(), &options).unwrap();

    // Should report 3 would be deleted
    assert_eq!(result.would_delete.len(), 3);

    // All files should still exist
    let remaining: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "log"))
        .collect();
    assert_eq!(remaining.len(), 5);
}

#[test]
fn cleanup_compress_creates_gz_files() {
    let dir = tempdir().unwrap();

    let log = dir.path().join("test.log");
    fs::write(&log, "test content for compression").unwrap();

    let options = CleanupOptions::new().delete_all(true).compress(true);
    let result = cleanup(dir.path(), &options).unwrap();

    assert_eq!(result.compressed.len(), 1);

    // Original should be gone
    assert!(!log.exists());

    // .gz should exist
    let gz = dir.path().join("test.log.gz");
    assert!(gz.exists());
}

#[test]
fn cleanup_compress_dry_run() {
    let dir = tempdir().unwrap();

    let log = dir.path().join("test.log");
    fs::write(&log, "test content for compression").unwrap();

    let options = CleanupOptions::new()
        .delete_all(true)
        .compress(true)
        .dry_run(true);
    let result = cleanup(dir.path(), &options).unwrap();

    assert_eq!(result.would_compress.len(), 1);
    assert!(result.would_compress_save > 0);

    // Original should still exist
    assert!(log.exists());

    // .gz should NOT exist
    let gz = dir.path().join("test.log.gz");
    assert!(!gz.exists());
}

#[test]
fn cleanup_options_builder_new_fields() {
    use chrono::NaiveDate;

    let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

    let options = CleanupOptions::new()
        .before_date(date)
        .after_date(date)
        .keep_last(5)
        .compress(true);

    assert_eq!(options.before_date, Some(date));
    assert_eq!(options.after_date, Some(date));
    assert_eq!(options.keep_last, Some(5));
    assert!(options.compress);
}
