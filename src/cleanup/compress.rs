//! File compression and directory cleanup utilities.

use crate::Error;
use flate2::Compression;
use flate2::write::GzEncoder;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

/// Compresses a file using gzip.
///
/// Returns the bytes saved (original size - compressed size).
pub(super) fn compress_file(path: &Path) -> Result<u64, Error> {
    let input = File::open(path)?;
    let original_size = input.metadata()?.len();
    let mut reader = BufReader::new(input);

    let gz_path = format!("{}.gz", path.display());
    let output = File::create(&gz_path)?;
    let writer = BufWriter::new(output);
    let mut encoder = GzEncoder::new(writer, Compression::default());

    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        encoder.write_all(&buffer[..bytes_read])?;
    }
    encoder.finish()?;

    // Get compressed size and calculate savings
    let compressed_size = fs::metadata(&gz_path)?.len();
    let saved = original_size.saturating_sub(compressed_size);

    // Remove original file
    fs::remove_file(path)?;

    Ok(saved)
}

/// Cleans up empty directories recursively.
pub(super) fn cleanup_empty_dirs(dir: &Path) -> Result<(), Error> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            cleanup_empty_dirs(&path)?;
            // Try to remove if empty (will fail if not empty)
            let _ = fs::remove_dir(&path);
        }
    }

    Ok(())
}
