//! Tracks outcomes of cleanup runs — split into actual vs dry-run results
//! so callers can report, undo, or preview without separate code paths.

use super::format_size;
use crate::logger::Logger;

/// The CLI needs structured data to show users what happened (or would happen in dry-run).
#[derive(Debug, Default)]
pub struct CleanupResult {
    /// Successfully removed files (used for reporting and potential undo tracking).
    pub deleted: Vec<String>,
    /// Users want to know how much disk they recovered.
    pub freed: u64,
    /// Dry run needs its own list because actual deletion hasn't happened.
    pub would_delete: Vec<String>,
    /// Dry run estimate so users can decide before committing.
    pub would_free: u64,
    /// Compression is tracked separately because it reclaims less space than deletion.
    pub compressed: Vec<String>,
    /// Users want to see compression savings separately from deletion savings.
    pub compressed_saved: u64,
    /// Dry run needs its own compression list because no .gz files were created yet.
    pub would_compress: Vec<String>,
    /// Rough estimate since actual compression ratio depends on content.
    pub would_compress_save: u64,
    /// Files that could not be processed, with the reason (for error reporting).
    pub failed: Vec<(String, String)>,
}

impl CleanupResult {
    /// Unifies actual and dry-run counts so callers don't need to branch on mode.
    #[must_use]
    pub const fn count(&self) -> usize {
        if self.deleted.is_empty() {
            self.would_delete.len()
        } else {
            self.deleted.len()
        }
    }

    /// Unifies actual and dry-run byte counts so callers don't need to branch on mode.
    #[must_use]
    pub const fn bytes(&self) -> u64 {
        if self.freed == 0 {
            self.would_free
        } else {
            self.freed
        }
    }

    /// Unifies actual and dry-run compression counts.
    #[must_use]
    pub const fn compressed_count(&self) -> usize {
        if self.compressed.is_empty() {
            self.would_compress.len()
        } else {
            self.compressed.len()
        }
    }

    /// Unifies actual and dry-run compression byte savings.
    #[must_use]
    pub const fn compressed_bytes(&self) -> u64 {
        if self.compressed_saved == 0 {
            self.would_compress_save
        } else {
            self.compressed_saved
        }
    }

    /// Users need confirmation of what the cleanup did (or would do).
    pub fn log(&self, logger: &Logger, dry_run: bool) {
        let has_output = if dry_run {
            self.log_dry_run(logger)
        } else {
            self.log_actual(logger)
        };

        if !has_output {
            let msg = if dry_run {
                "No files to process"
            } else {
                "No files processed"
            };
            logger.debug("CLEANUP", msg);
        }
    }

    fn log_dry_run(&self, logger: &Logger) -> bool {
        if !self.would_delete.is_empty() {
            let count = self.would_delete.len();
            let size = format_size(self.would_free);
            logger.print(
                "CLEANUP",
                &format!("Would delete {count} file(s), freeing {size}"),
            );
            for path in &self.would_delete {
                logger.raw(&format!("  {path}"));
            }
        }

        if !self.would_compress.is_empty() {
            let count = self.would_compress.len();
            let size = format_size(self.would_compress_save);
            logger.print(
                "CLEANUP",
                &format!("Would compress {count} file(s), saving ~{size}"),
            );
            for path in &self.would_compress {
                logger.raw(&format!("  {path}"));
            }
        }

        !self.would_delete.is_empty() || !self.would_compress.is_empty()
    }

    fn log_actual(&self, logger: &Logger) -> bool {
        if !self.deleted.is_empty() {
            let count = self.deleted.len();
            let size = format_size(self.freed);
            logger.print("CLEANUP", &format!("Deleted {count} file(s), freed {size}"));
            for path in &self.deleted {
                logger.raw(&format!("  {path}"));
            }
        }

        if !self.compressed.is_empty() {
            let count = self.compressed.len();
            let size = format_size(self.compressed_saved);
            logger.print(
                "CLEANUP",
                &format!("Compressed {count} file(s), saved {size}"),
            );
            for path in &self.compressed {
                logger.raw(&format!("  {path}"));
            }
        }

        !self.deleted.is_empty() || !self.compressed.is_empty()
    }
}
