//! Log file statistics types.

use super::format_size;
use crate::logger::Logger;
use chrono::NaiveDate;

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

impl LogStats {
    /// Prints the statistics using the provided logger.
    ///
    /// Uses `print()` to bypass level filtering - command output should
    /// always be visible regardless of configured log level.
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
