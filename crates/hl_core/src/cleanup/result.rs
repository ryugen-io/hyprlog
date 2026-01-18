//! Cleanup operation result types.

use super::format_size;
use crate::logger::Logger;

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

    /// Logs the cleanup result using the provided logger.
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
            logger.info(
                "CLEANUP",
                &format!("Would delete {count} file(s), freeing {size}"),
            );
            for path in &self.would_delete {
                logger.info("CLEANUP", &format!("  {path}"));
            }
        }

        if !self.would_compress.is_empty() {
            let count = self.would_compress.len();
            let size = format_size(self.would_compress_save);
            logger.info(
                "CLEANUP",
                &format!("Would compress {count} file(s), saving ~{size}"),
            );
            for path in &self.would_compress {
                logger.info("CLEANUP", &format!("  {path}"));
            }
        }

        !self.would_delete.is_empty() || !self.would_compress.is_empty()
    }

    fn log_actual(&self, logger: &Logger) -> bool {
        if !self.deleted.is_empty() {
            let count = self.deleted.len();
            let size = format_size(self.freed);
            logger.info("CLEANUP", &format!("Deleted {count} file(s), freed {size}"));
            for path in &self.deleted {
                logger.info("CLEANUP", &format!("  {path}"));
            }
        }

        if !self.compressed.is_empty() {
            let count = self.compressed.len();
            let size = format_size(self.compressed_saved);
            logger.info(
                "CLEANUP",
                &format!("Compressed {count} file(s), saved {size}"),
            );
            for path in &self.compressed {
                logger.info("CLEANUP", &format!("  {path}"));
            }
        }

        !self.deleted.is_empty() || !self.compressed.is_empty()
    }
}
