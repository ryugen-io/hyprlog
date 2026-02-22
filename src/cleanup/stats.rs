//! The stats command needs structured data to display — these types
//! carry the metadata from filesystem scan to CLI rendering.

use super::format_size;
use crate::logger::Logger;
use chrono::NaiveDate;

/// Cleanup and stats both need the same directory scan, so this carries the shared result.
#[derive(Debug, Default)]
pub struct LogStats {
    /// Count of discovered `.log` files — shown as the headline stat.
    pub total_files: usize,
    /// Combined byte size of all files — used to gauge disk pressure.
    pub total_size: u64,
    /// Path to the file with the highest age — shows how far back retention reaches.
    pub oldest_file: Option<String>,
    /// Path to the most recently modified file — confirms logging is still active.
    pub newest_file: Option<String>,
    /// Per-file metadata for the detailed file listing.
    pub files: Vec<LogFileInfo>,
}

impl LogStats {
    /// Uses `print()` to bypass level filtering — command output should always
    /// be visible regardless of the configured minimum log level.
    pub fn log(&self, logger: &Logger) {
        logger.print("STATS", &format!("Total files: {}", self.total_files));
        logger.print(
            "STATS",
            &format!("Total size:  {}", format_size(self.total_size)),
        );

        if let Some(oldest) = &self.oldest_file {
            logger.print("STATS", &format!("Oldest:      {oldest}"));
        }
        if let Some(newest) = &self.newest_file {
            logger.print("STATS", &format!("Newest:      {newest}"));
        }

        if !self.files.is_empty() {
            logger.print("STATS", "Files:");
            for file in &self.files {
                let age = if file.age_days == 0 {
                    "today".to_string()
                } else if file.age_days == 1 {
                    "1 day".to_string()
                } else {
                    format!("{} days", file.age_days)
                };
                logger.raw(&format!(
                    "  {} ({}, {})",
                    file.path,
                    format_size(file.size),
                    age
                ));
            }
        }
    }
}

/// Cleanup and stats both need the same per-file metadata, so one struct serves both.
#[derive(Debug, Clone)]
pub struct LogFileInfo {
    /// Absolute path — serves as the unique identity for protection and dedup checks.
    pub path: String,
    /// Byte size on disk — compared against `max_total_size` for eviction.
    pub size: u64,
    /// Days since last modification — compared against `max_age_days` for expiry.
    pub age_days: u64,
    /// Calendar date of last modification — used by `before_date`/`after_date` filters.
    pub modified_date: Option<NaiveDate>,
}
