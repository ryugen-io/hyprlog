//! Log file cleanup and statistics.

mod compress;
mod error;
mod files;
mod options;
mod result;
mod size;
mod stats;

pub use error::CleanupError;
pub use options::CleanupOptions;
pub use result::CleanupResult;
pub use size::{format_size, parse_size};
pub use stats::{LogFileInfo, LogStats};

use crate::internal;
use compress::{cleanup_empty_dirs, compress_file};
use files::collect_log_files;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

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
