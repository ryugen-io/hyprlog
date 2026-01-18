//! Cleanup operation result types.

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
}
