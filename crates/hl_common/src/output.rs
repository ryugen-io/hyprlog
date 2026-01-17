//! Output formatting for CLI results.

use hl_core::{CleanupResult, LogStats, format_size};

/// Formatter for CLI output.
#[derive(Debug, Default)]
pub struct OutputFormatter {
    /// Enable colored output.
    pub colors: bool,
}

impl OutputFormatter {
    /// Creates a new formatter.
    #[must_use]
    pub const fn new() -> Self {
        Self { colors: false }
    }

    /// Enables colored output.
    #[must_use]
    pub const fn colors(mut self, enabled: bool) -> Self {
        self.colors = enabled;
        self
    }

    /// Formats log statistics.
    #[must_use]
    pub fn format_stats(&self, stats: &LogStats) -> String {
        let mut lines = Vec::new();

        lines.push(format!("Total files: {}", stats.total_files));
        lines.push(format!("Total size:  {}", format_size(stats.total_size)));

        if let Some(oldest) = &stats.oldest_file {
            lines.push(format!("Oldest:      {oldest}"));
        }
        if let Some(newest) = &stats.newest_file {
            lines.push(format!("Newest:      {newest}"));
        }

        if !stats.files.is_empty() {
            lines.push(String::new());
            lines.push("Files:".to_string());
            for file in &stats.files {
                let age = if file.age_days == 0 {
                    "today".to_string()
                } else if file.age_days == 1 {
                    "1 day".to_string()
                } else {
                    format!("{} days", file.age_days)
                };
                lines.push(format!(
                    "  {} ({}, {})",
                    file.path,
                    format_size(file.size),
                    age
                ));
            }
        }

        lines.join("\n")
    }

    /// Formats cleanup results.
    #[must_use]
    pub fn format_cleanup(&self, result: &CleanupResult, dry_run: bool) -> String {
        let mut lines = Vec::new();

        if dry_run {
            // Deletion info
            if !result.would_delete.is_empty() {
                let count = result.would_delete.len();
                let size = format_size(result.would_free);
                lines.push(format!("Would delete {count} file(s), freeing {size}"));
                lines.push(String::new());
                for path in &result.would_delete {
                    lines.push(format!("  {path}"));
                }
            }

            // Compression info
            if !result.would_compress.is_empty() {
                if !lines.is_empty() {
                    lines.push(String::new());
                }
                let count = result.would_compress.len();
                let size = format_size(result.would_compress_save);
                lines.push(format!("Would compress {count} file(s), saving ~{size}"));
                lines.push(String::new());
                for path in &result.would_compress {
                    lines.push(format!("  {path}"));
                }
            }

            if lines.is_empty() {
                lines.push("No files to process".to_string());
            }
        } else {
            // Deletion info
            if !result.deleted.is_empty() {
                let count = result.deleted.len();
                let size = format_size(result.freed);
                lines.push(format!("Deleted {count} file(s), freed {size}"));
                lines.push(String::new());
                for path in &result.deleted {
                    lines.push(format!("  {path}"));
                }
            }

            // Compression info
            if !result.compressed.is_empty() {
                if !lines.is_empty() {
                    lines.push(String::new());
                }
                let count = result.compressed.len();
                let size = format_size(result.compressed_saved);
                lines.push(format!("Compressed {count} file(s), saved {size}"));
                lines.push(String::new());
                for path in &result.compressed {
                    lines.push(format!("  {path}"));
                }
            }

            if lines.is_empty() {
                lines.push("No files processed".to_string());
            }
        }

        lines.join("\n")
    }
}
