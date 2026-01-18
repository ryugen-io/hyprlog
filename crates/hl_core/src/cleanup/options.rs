//! Cleanup options and configuration.

use super::size::parse_size;
use chrono::NaiveDate;

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
