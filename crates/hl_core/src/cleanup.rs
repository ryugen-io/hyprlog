//! Log file cleanup and statistics.

use crate::internal;
use chrono::NaiveDate;
use flate2::Compression;
use flate2::write::GzEncoder;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::time::SystemTime;

/// Error type for cleanup operations.
#[derive(Debug)]
pub enum CleanupError {
    /// I/O error.
    Io(std::io::Error),
    /// Invalid path.
    InvalidPath(String),
}

impl std::fmt::Display for CleanupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::InvalidPath(s) => write!(f, "invalid path: {s}"),
        }
    }
}

impl std::error::Error for CleanupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::InvalidPath(_) => None,
        }
    }
}

impl From<std::io::Error> for CleanupError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// Options for cleanup operations.
#[derive(Debug, Clone, Default)]
pub struct CleanupOptions {
    /// Maximum age in days (None = no age limit).
    pub max_age_days: Option<u32>,
    /// Maximum total size in bytes (None = no size limit).
    pub max_total_size: Option<u64>,
    /// Filter by app name (None = all apps).
    pub app_filter: Option<String>,
    /// Delete ALL files.
    pub delete_all: bool,
    /// Dry run - report but don't delete.
    pub dry_run: bool,
    /// Delete files modified before this date.
    pub before_date: Option<NaiveDate>,
    /// Delete files modified after this date.
    pub after_date: Option<NaiveDate>,
    /// Always keep the N most recent files.
    pub keep_last: Option<usize>,
    /// Compress files instead of deleting.
    pub compress: bool,
}

impl CleanupOptions {
    /// Creates new cleanup options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets maximum age in days.
    #[must_use]
    pub const fn max_age_days(mut self, days: u32) -> Self {
        self.max_age_days = Some(days);
        self
    }

    /// Sets maximum total size from a string like "500M" or "1G".
    #[must_use]
    pub fn max_total_size(mut self, size: &str) -> Self {
        self.max_total_size = parse_size(size);
        self
    }

    /// Sets maximum total size in bytes.
    #[must_use]
    pub const fn max_total_size_bytes(mut self, bytes: u64) -> Self {
        self.max_total_size = Some(bytes);
        self
    }

    /// Sets app filter.
    #[must_use]
    pub fn app_filter(mut self, app: impl Into<String>) -> Self {
        self.app_filter = Some(app.into());
        self
    }

    /// Sets delete all flag.
    #[must_use]
    pub const fn delete_all(mut self, delete: bool) -> Self {
        self.delete_all = delete;
        self
    }

    /// Sets dry run flag.
    #[must_use]
    pub const fn dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    /// Sets before date filter (delete files modified before this date).
    #[must_use]
    pub const fn before_date(mut self, date: NaiveDate) -> Self {
        self.before_date = Some(date);
        self
    }

    /// Sets after date filter (delete files modified after this date).
    #[must_use]
    pub const fn after_date(mut self, date: NaiveDate) -> Self {
        self.after_date = Some(date);
        self
    }

    /// Sets keep last N files.
    #[must_use]
    pub const fn keep_last(mut self, n: usize) -> Self {
        self.keep_last = Some(n);
        self
    }

    /// Sets compress flag (compress instead of delete).
    #[must_use]
    pub const fn compress(mut self, compress: bool) -> Self {
        self.compress = compress;
        self
    }
}

/// Result of a cleanup operation.
#[derive(Debug, Default)]
pub struct CleanupResult {
    /// Files that were deleted.
    pub deleted: Vec<String>,
    /// Bytes freed.
    pub freed: u64,
    /// Files that would be deleted (dry run).
    pub would_delete: Vec<String>,
    /// Bytes that would be freed (dry run).
    pub would_free: u64,
    /// Files that were compressed.
    pub compressed: Vec<String>,
    /// Bytes saved by compression.
    pub compressed_saved: u64,
    /// Files that would be compressed (dry run).
    pub would_compress: Vec<String>,
    /// Bytes that would be saved (dry run estimate).
    pub would_compress_save: u64,
    /// Files that failed to process (path, error message).
    pub failed: Vec<(String, String)>,
}

impl CleanupResult {
    /// Returns the number of files deleted.
    #[must_use]
    pub fn count(&self) -> usize {
        if self.deleted.is_empty() {
            self.would_delete.len()
        } else {
            self.deleted.len()
        }
    }

    /// Returns the bytes freed by deletion.
    #[must_use]
    pub const fn bytes(&self) -> u64 {
        if self.freed == 0 {
            self.would_free
        } else {
            self.freed
        }
    }

    /// Returns the number of files compressed.
    #[must_use]
    pub fn compressed_count(&self) -> usize {
        if self.compressed.is_empty() {
            self.would_compress.len()
        } else {
            self.compressed.len()
        }
    }

    /// Returns the bytes saved by compression.
    #[must_use]
    pub const fn compressed_bytes(&self) -> u64 {
        if self.compressed_saved == 0 {
            self.would_compress_save
        } else {
            self.compressed_saved
        }
    }
}

/// Statistics about log files.
#[derive(Debug, Default)]
pub struct LogStats {
    /// Total number of log files.
    pub total_files: usize,
    /// Total size in bytes.
    pub total_size: u64,
    /// Oldest file path.
    pub oldest_file: Option<String>,
    /// Newest file path.
    pub newest_file: Option<String>,
    /// Files with (path, size, `age_days`).
    pub files: Vec<LogFileInfo>,
}

/// Information about a log file.
#[derive(Debug, Clone)]
pub struct LogFileInfo {
    /// File path.
    pub path: String,
    /// File size in bytes.
    pub size: u64,
    /// Age in days.
    pub age_days: u64,
    /// Modification date.
    pub modified_date: Option<NaiveDate>,
}

/// Performs cleanup on log files.
///
/// # Errors
/// Returns error if cleanup fails.
#[allow(clippy::too_many_lines)]
pub fn cleanup(base_dir: &Path, options: &CleanupOptions) -> Result<CleanupResult, CleanupError> {
    internal::info(
        "CLEANUP",
        &format!("Starting cleanup in {}", base_dir.display()),
    );
    internal::debug(
        "CLEANUP",
        &format!(
            "Options: delete_all={}, dry_run={}, compress={}",
            options.delete_all, options.dry_run, options.compress
        ),
    );

    let mut result = CleanupResult::default();
    let now = SystemTime::now();

    if !base_dir.exists() {
        internal::debug("CLEANUP", "Base directory does not exist, nothing to clean");
        return Ok(result);
    }

    // Collect all log files
    let mut files = collect_log_files(base_dir, now, options.app_filter.as_deref())?;

    // Sort by age (oldest first for deletion, newest first for keep_last)
    files.sort_by_key(|f| std::cmp::Reverse(f.age_days));

    // Track which files to skip (keep_last protection)
    let mut protected_paths: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Apply keep_last: protect the N most recent files
    if let Some(keep_n) = options.keep_last {
        internal::debug("CLEANUP", &format!("Protecting {keep_n} most recent files"));
        // Sort by age ascending (newest first) to find files to protect
        let mut by_newest = files.clone();
        by_newest.sort_by_key(|f| f.age_days);
        for file in by_newest.iter().take(keep_n) {
            protected_paths.insert(file.path.clone());
        }
    }

    // Determine which files should be processed
    for file in &files {
        // Skip protected files
        if protected_paths.contains(&file.path) {
            continue;
        }

        let mut should_process = options.delete_all;

        // Check age filter
        if let Some(max) = options.max_age_days {
            if file.age_days > u64::from(max) {
                internal::trace("CLEANUP", &format!("File {} exceeds age limit", file.path));
                should_process = true;
            }
        }

        // Check before_date filter
        if let Some(before) = options.before_date {
            if let Some(mod_date) = file.modified_date {
                if mod_date < before {
                    should_process = true;
                }
            }
        }

        // Check after_date filter
        if let Some(after) = options.after_date {
            if let Some(mod_date) = file.modified_date {
                if mod_date > after {
                    should_process = true;
                }
            }
        }

        if should_process {
            if options.compress {
                // Compress instead of delete
                if options.dry_run {
                    result.would_compress.push(file.path.clone());
                    // Estimate ~50% compression ratio for text logs
                    result.would_compress_save += file.size / 2;
                } else {
                    internal::debug("CLEANUP", &format!("Compressing: {}", file.path));
                    let path = Path::new(&file.path);
                    match compress_file(path) {
                        Ok(saved) => {
                            #[allow(clippy::cast_precision_loss)]
                            let pct = if file.size > 0 {
                                (saved as f64 / file.size as f64) * 100.0
                            } else {
                                0.0
                            };
                            let new_size = file.size.saturating_sub(saved);
                            internal::debug(
                                "CLEANUP",
                                &format!(
                                    "Compressed {}: {} -> {new_size} ({pct:.0}% saved)",
                                    file.path, file.size
                                ),
                            );
                            result.compressed.push(file.path.clone());
                            result.compressed_saved += saved;
                        }
                        Err(e) => {
                            result.failed.push((file.path.clone(), e.to_string()));
                        }
                    }
                }
            } else {
                // Delete
                if options.dry_run {
                    result.would_delete.push(file.path.clone());
                    result.would_free += file.size;
                } else {
                    internal::debug("CLEANUP", &format!("Deleting: {}", file.path));
                    if fs::remove_file(&file.path).is_ok() {
                        result.deleted.push(file.path.clone());
                        result.freed += file.size;
                    }
                }
            }
        }
    }

    // Delete by size limit (only applies to deletion, not compression)
    if !options.compress {
        if let Some(limit) = options.max_total_size {
            let remaining: Vec<_> = files
                .iter()
                .filter(|f| {
                    !result.deleted.contains(&f.path)
                        && !result.would_delete.contains(&f.path)
                        && !protected_paths.contains(&f.path)
                })
                .collect();

            let mut total: u64 = remaining.iter().map(|f| f.size).sum();

            // Delete oldest files until under limit
            for file in remaining.iter().rev() {
                if total <= limit {
                    break;
                }
                if options.dry_run {
                    result.would_delete.push(file.path.clone());
                    result.would_free += file.size;
                } else if fs::remove_file(&file.path).is_ok() {
                    result.deleted.push(file.path.clone());
                    result.freed += file.size;
                }
                total = total.saturating_sub(file.size);
            }
        }
    }

    // Clean up empty directories
    if !options.dry_run {
        cleanup_empty_dirs(base_dir)?;
    }

    let total_count = result.count() + result.compressed_count();
    let total_bytes = result.bytes() + result.compressed_bytes();
    internal::info(
        "CLEANUP",
        &format!("Cleanup complete: {total_count} files, {total_bytes} bytes freed"),
    );

    Ok(result)
}

/// Gets statistics about log files.
///
/// # Errors
/// Returns error if stats cannot be collected.
pub fn stats(base_dir: &Path, app_filter: Option<&str>) -> Result<LogStats, CleanupError> {
    let mut stats = LogStats::default();
    let now = SystemTime::now();

    if !base_dir.exists() {
        return Ok(stats);
    }

    let files = collect_log_files(base_dir, now, app_filter)?;

    stats.total_files = files.len();
    stats.total_size = files.iter().map(|f| f.size).sum();

    if let Some(oldest) = files.iter().max_by_key(|f| f.age_days) {
        stats.oldest_file = Some(oldest.path.clone());
    }
    if let Some(newest) = files.iter().min_by_key(|f| f.age_days) {
        stats.newest_file = Some(newest.path.clone());
    }

    stats.files = files;

    Ok(stats)
}

fn collect_log_files(
    dir: &Path,
    now: SystemTime,
    app_filter: Option<&str>,
) -> Result<Vec<LogFileInfo>, CleanupError> {
    internal::debug(
        "CLEANUP",
        &format!("Collecting log files from {}", dir.display()),
    );
    let mut files = Vec::new();
    collect_log_files_recursive(dir, now, app_filter, &mut files)?;
    internal::debug("CLEANUP", &format!("Found {} log files", files.len()));
    Ok(files)
}

fn collect_log_files_recursive(
    dir: &Path,
    now: SystemTime,
    app_filter: Option<&str>,
    files: &mut Vec<LogFileInfo>,
) -> Result<(), CleanupError> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(app) = app_filter {
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if dir_name == app {
                    // Found app dir, collect all files within
                    collect_log_files_recursive(&path, now, None, files)?;
                } else {
                    // Keep searching
                    collect_log_files_recursive(&path, now, app_filter, files)?;
                }
            } else {
                collect_log_files_recursive(&path, now, None, files)?;
            }
        } else if app_filter.is_none() && path.extension().is_some_and(|e| e == "log") {
            if let Ok(meta) = fs::metadata(&path) {
                let size = meta.len();
                let modified = meta.modified().ok();
                let age_days = modified
                    .and_then(|m| now.duration_since(m).ok())
                    .map_or(0, |d| d.as_secs() / 86400);

                let modified_date = modified.and_then(|m| {
                    let duration = m.duration_since(std::time::UNIX_EPOCH).ok()?;
                    let timestamp = i64::try_from(duration.as_secs()).ok()?;
                    chrono::DateTime::from_timestamp(timestamp, 0).map(|dt| dt.naive_utc().date())
                });

                internal::trace("CLEANUP", &format!("Found: {}", path.display()));
                files.push(LogFileInfo {
                    path: path.display().to_string(),
                    size,
                    age_days,
                    modified_date,
                });
            }
        }
    }

    Ok(())
}

/// Compresses a file using gzip.
///
/// Returns the bytes saved (original size - compressed size).
fn compress_file(path: &Path) -> Result<u64, CleanupError> {
    let input = File::open(path)?;
    let original_size = input.metadata()?.len();
    let mut reader = BufReader::new(input);

    let gz_path = format!("{}.gz", path.display());
    let output = File::create(&gz_path)?;
    let writer = BufWriter::new(output);
    let mut encoder = GzEncoder::new(writer, Compression::default());

    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        encoder.write_all(&buffer[..bytes_read])?;
    }
    encoder.finish()?;

    // Get compressed size and calculate savings
    let compressed_size = fs::metadata(&gz_path)?.len();
    let saved = original_size.saturating_sub(compressed_size);

    // Remove original file
    fs::remove_file(path)?;

    Ok(saved)
}

fn cleanup_empty_dirs(dir: &Path) -> Result<(), CleanupError> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            cleanup_empty_dirs(&path)?;
            // Try to remove if empty (will fail if not empty)
            let _ = fs::remove_dir(&path);
        }
    }

    Ok(())
}

/// Parses a size string like "500M" or "1G" to bytes.
#[must_use]
pub fn parse_size(s: &str) -> Option<u64> {
    let s = s.trim().to_uppercase();
    let (num_str, multiplier): (&str, f64) = if s.ends_with("GB") || s.ends_with('G') {
        (
            s.trim_end_matches("GB").trim_end_matches('G'),
            1024.0 * 1024.0 * 1024.0,
        )
    } else if s.ends_with("MB") || s.ends_with('M') {
        (
            s.trim_end_matches("MB").trim_end_matches('M'),
            1024.0 * 1024.0,
        )
    } else if s.ends_with("KB") || s.ends_with('K') {
        (s.trim_end_matches("KB").trim_end_matches('K'), 1024.0)
    } else {
        (s.as_str(), 1.0)
    };

    num_str.trim().parse::<f64>().ok().map(|n| {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let result = (n * multiplier) as u64;
        result
    })
}

/// Formats bytes as a human-readable string.
#[must_use]
pub fn format_size(bytes: u64) -> String {
    #[allow(clippy::cast_precision_loss)]
    let bytes_f = bytes as f64;

    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.2} GB", bytes_f / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.2} MB", bytes_f / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.2} KB", bytes_f / 1024.0)
    } else {
        format!("{bytes} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
