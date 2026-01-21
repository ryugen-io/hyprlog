//! Log file collection utilities.

use super::error::CleanupError;
use super::stats::LogFileInfo;
use crate::internal;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

/// Collects all log files from the directory.
pub(super) fn collect_log_files(
    dir: &Path,
    now: SystemTime,
    app_filter: Option<&str>,
) -> Result<Vec<LogFileInfo>, CleanupError> {
    internal::debug(
        "CLEANUP",
        &format!("Collecting log files from {}", dir.display()),
    );
    let mut files = Vec::new();
    let mut folders = HashSet::new();
    collect_log_files_recursive(dir, now, app_filter, &mut files, &mut folders)?;
    internal::debug(
        "CLEANUP",
        &format!(
            "Found {} log files in {} folders",
            files.len(),
            folders.len()
        ),
    );
    Ok(files)
}

fn collect_log_files_recursive(
    dir: &Path,
    now: SystemTime,
    app_filter: Option<&str>,
    files: &mut Vec<LogFileInfo>,
    folders: &mut HashSet<String>,
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
                    collect_log_files_recursive(&path, now, None, files, folders)?;
                } else {
                    // Keep searching
                    collect_log_files_recursive(&path, now, app_filter, files, folders)?;
                }
            } else {
                collect_log_files_recursive(&path, now, None, files, folders)?;
            }
        } else if app_filter.is_none()
            && path.extension().is_some_and(|e| e == "log")
            && let Ok(meta) = fs::metadata(&path)
        {
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

            // Track parent folder
            if let Some(parent) = path.parent() {
                folders.insert(parent.display().to_string());
            }

            internal::trace("CLEANUP", &format!("Found: {}", path.display()));
            files.push(LogFileInfo {
                path: path.display().to_string(),
                size,
                age_days,
                modified_date,
            });
        }
    }

    Ok(())
}
