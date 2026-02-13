//! JSON output for structured log database.

use super::{LogRecord, Output};
use crate::fmt::style;
use crate::internal;

use chrono::Local;
use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use ulid::Ulid;

/// A single JSON log entry for the database.
#[derive(Debug, Serialize)]
struct JsonEntry {
    /// Unique ULID identifier (sortable, time-based).
    id: String,
    /// ISO 8601 timestamp.
    ts: String,
    /// Log level.
    level: String,
    /// Scope/module.
    scope: String,
    /// Log message (without styling tags).
    msg: String,
    /// Application name.
    #[serde(skip_serializing_if = "Option::is_none")]
    app: Option<String>,
    /// Custom label override (e.g., "SUCCESS" instead of "INFO").
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
}

/// JSON Lines output configuration.
///
/// Writes log entries as JSON Lines (one JSON object per line) to a file,
/// creating a queryable log database.
#[derive(Debug, Clone)]
pub struct JsonOutput {
    /// Path to the JSONL file.
    file_path: PathBuf,
    /// Application name for entries.
    app_name: Option<String>,
}

impl Default for JsonOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonOutput {
    /// Creates a new JSON output with default path.
    ///
    /// Default location: `~/.local/state/hyprlog/db/hyprlog.jsonl`
    #[must_use]
    pub fn new() -> Self {
        let file_path = directories::ProjectDirs::from("", "", "hyprlog").map_or_else(
            || PathBuf::from("hyprlog.jsonl"),
            |dirs| {
                dirs.state_dir()
                    .unwrap_or_else(|| dirs.data_dir())
                    .join("db")
                    .join("hyprlog.jsonl")
            },
        );

        Self {
            file_path,
            app_name: None,
        }
    }

    /// Sets the output file path.
    #[must_use]
    pub fn path(mut self, path: impl Into<PathBuf>) -> Self {
        self.file_path = path.into();
        self
    }

    /// Sets the application name.
    #[must_use]
    pub fn app_name(mut self, name: impl Into<String>) -> Self {
        self.app_name = Some(name.into());
        self
    }

    /// Resolves the file path (expands ~).
    fn resolve_path(&self) -> PathBuf {
        let path_str = self.file_path.to_string_lossy();
        let expanded = shellexpand::tilde(&path_str);
        let path = PathBuf::from(expanded.as_ref());
        internal::trace("JSON", &format!("Resolved path: {}", path.display()));
        path
    }

    /// Creates a JSON entry from a log record.
    fn create_entry(&self, record: &LogRecord) -> JsonEntry {
        let now = Local::now();
        let clean_msg = style::strip_tags(&record.message);
        let app = record.app_name.clone().or_else(|| self.app_name.clone());

        JsonEntry {
            id: Ulid::new().to_string(),
            ts: now.to_rfc3339(),
            level: record.level.as_str().to_string(),
            scope: record.scope.clone(),
            msg: clean_msg,
            app,
            label: record.label_override.clone(),
        }
    }
}

impl Output for JsonOutput {
    fn write(&self, record: &LogRecord) -> Result<(), crate::Error> {
        // Skip raw messages (they're typically continuation/formatting lines)
        if record.raw {
            return Ok(());
        }

        let path = self.resolve_path();
        internal::trace("JSON", &format!("Writing to: {}", path.display()));

        // Create parent directories
        if let Some(parent) = path.parent()
            && !parent.exists()
        {
            match fs::create_dir_all(parent) {
                Ok(()) => {
                    internal::debug("JSON", &format!("Created directory: {}", parent.display()));
                }
                Err(e) => {
                    internal::error(
                        "JSON",
                        &format!("Failed to create directory {}: {}", parent.display(), e),
                    );
                    return Err(e.into());
                }
            }
        }

        // Create JSON entry
        let entry = self.create_entry(record);
        let json = serde_json::to_string(&entry)
            .map_err(|e| crate::Error::Format(format!("JSON serialization failed: {e}")))?;

        // Append to file (JSONL format: one JSON object per line)
        let mut file = OpenOptions::new().create(true).append(true).open(&path)?;

        writeln!(file, "{json}")?;

        Ok(())
    }

    fn flush(&self) -> Result<(), crate::Error> {
        Ok(())
    }
}
