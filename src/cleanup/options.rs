//! Separated from the cleanup engine so callers can construct policies
//! without importing filesystem internals.

use super::size::parse_size;
use chrono::NaiveDate;

/// All filters default to off so nothing gets deleted without explicit opt-in.
#[derive(Debug, Clone, Default)]
pub struct CleanupOptions {
    /// Prevents unbounded log growth by expiring old files.
    pub max_age_days: Option<u32>,
    /// Caps disk usage when time-based expiry alone isn't enough.
    pub max_total_size: Option<u64>,
    /// Multi-app setups need per-app cleanup to avoid collateral damage.
    pub app_filter: Option<String>,
    /// Escape hatch for "nuke everything" without configuring filters.
    pub delete_all: bool,
    /// Destructive operations need a preview mode to avoid accidents.
    pub dry_run: bool,
    /// Archive cleanup needs a date cutoff, not just an age in days.
    pub before_date: Option<NaiveDate>,
    /// Targeted removal of logs from a specific incident period.
    pub after_date: Option<NaiveDate>,
    /// Aggressive filters shouldn't delete the most recent diagnostics.
    pub keep_last: Option<usize>,
    /// Some users must retain logs for compliance but can't spare the disk space.
    pub compress: bool,
}

impl CleanupOptions {
    /// Defaults are safe — nothing gets processed until a filter is enabled.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Prevents unbounded log growth by expiring files past an age threshold.
    #[must_use]
    pub const fn max_age_days(mut self, days: u32) -> Self {
        self.max_age_days = Some(days);
        self
    }

    /// Config files use "500M"/"1G" notation, not raw byte counts.
    #[must_use]
    pub fn max_total_size(mut self, size: &str) -> Self {
        self.max_total_size = parse_size(size);
        self
    }

    /// Programmatic callers already have the threshold as bytes.
    #[must_use]
    pub const fn max_total_size_bytes(mut self, bytes: u64) -> Self {
        self.max_total_size = Some(bytes);
        self
    }

    /// Multi-app setups need per-app cleanup to avoid collateral damage.
    #[must_use]
    pub fn app_filter(mut self, app: impl Into<String>) -> Self {
        self.app_filter = Some(app.into());
        self
    }

    /// Escape hatch for "nuke everything" without configuring individual filters.
    #[must_use]
    pub const fn delete_all(mut self, delete: bool) -> Self {
        self.delete_all = delete;
        self
    }

    /// Destructive operations need a preview mode to build user trust.
    #[must_use]
    pub const fn dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    /// Archive cleanup needs a date cutoff, not just an age in days.
    #[must_use]
    pub const fn before_date(mut self, date: NaiveDate) -> Self {
        self.before_date = Some(date);
        self
    }

    /// Targeted removal of logs from a specific incident period.
    #[must_use]
    pub const fn after_date(mut self, date: NaiveDate) -> Self {
        self.after_date = Some(date);
        self
    }

    /// Aggressive filters shouldn't delete the most recent diagnostic data.
    #[must_use]
    pub const fn keep_last(mut self, n: usize) -> Self {
        self.keep_last = Some(n);
        self
    }

    /// Some users must retain logs for compliance but can't spare the disk space.
    #[must_use]
    pub const fn compress(mut self, compress: bool) -> Self {
        self.compress = compress;
        self
    }
}
