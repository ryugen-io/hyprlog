//! Log file statistics types.

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
