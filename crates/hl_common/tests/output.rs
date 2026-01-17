//! Tests for output formatting functionality.

use hl_common::OutputFormatter;
use hl_core::{CleanupResult, LogFileInfo, LogStats};

#[test]
fn format_empty_stats() {
    let formatter = OutputFormatter::new();
    let stats = LogStats::default();
    let output = formatter.format_stats(&stats);
    assert!(output.contains("Total files: 0"));
    assert!(output.contains("Total size:  0 B"));
}

#[test]
fn format_stats_with_files() {
    let formatter = OutputFormatter::new();
    let stats = LogStats {
        total_files: 2,
        total_size: 2048,
        oldest_file: Some("/old.log".to_string()),
        newest_file: Some("/new.log".to_string()),
        files: vec![
            LogFileInfo {
                path: "/old.log".to_string(),
                size: 1024,
                age_days: 5,
                modified_date: None,
            },
            LogFileInfo {
                path: "/new.log".to_string(),
                size: 1024,
                age_days: 0,
                modified_date: None,
            },
        ],
    };
    let output = formatter.format_stats(&stats);
    assert!(output.contains("Total files: 2"));
    assert!(output.contains("5 days"));
    assert!(output.contains("today"));
}

#[test]
fn format_cleanup_dry_run() {
    let formatter = OutputFormatter::new();
    let result = CleanupResult {
        would_delete: vec!["/test.log".to_string()],
        would_free: 1024,
        ..Default::default()
    };
    let output = formatter.format_cleanup(&result, true);
    assert!(output.contains("Would delete 1 file(s)"));
    assert!(output.contains("/test.log"));
}

#[test]
fn format_cleanup_actual() {
    let formatter = OutputFormatter::new();
    let result = CleanupResult {
        deleted: vec!["/test.log".to_string()],
        freed: 1024,
        ..Default::default()
    };
    let output = formatter.format_cleanup(&result, false);
    assert!(output.contains("Deleted 1 file(s)"));
}
