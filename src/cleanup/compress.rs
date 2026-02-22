//! Deleting old logs loses data permanently — gzip compression reclaims most of the
//! disk space while keeping content available for future forensics.

use crate::Error;
use flate2::Compression;
use flate2::write::GzEncoder;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

/// In-place compression (create .gz, remove original) avoids needing temporary storage
/// for the entire uncompressed file. Returns bytes saved so callers can report totals.
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

    // Compare sizes after compression to report how much disk space was reclaimed
    let compressed_size = fs::metadata(&gz_path)?.len();
    let saved = original_size.saturating_sub(compressed_size);

    // The .gz now holds all content, so remove the original to actually free space
    fs::remove_file(path)?;

    Ok(saved)
}

/// After deleting/compressing files, empty parent directories litter the tree —
/// removing them prevents `ls` from showing ghost directories with no content.
pub(super) fn cleanup_empty_dirs(dir: &Path) -> Result<(), Error> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            cleanup_empty_dirs(&path)?;
            // Attempt removal after recursing children; silently fails if non-empty, which is expected
            let _ = fs::remove_dir(&path);
        }
    }

    Ok(())
}
