//! Plain log files can't be efficiently queried for aggregates — JSONL gives `hyprlog stats`
//! a structured database without requiring `SQLite` or a separate service.

use super::{LogRecord, Output};
use crate::fmt::style;
use crate::internal;

use chrono::Local;
use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use ulid::Ulid;

/// Flat structure optimized for JSONL — one object per line enables `grep`, `jq`, and `hyprlog stats` queries.
#[derive(Debug, Serialize)]
struct JsonEntry {
    /// ULID is time-sortable and globally unique — no collisions even with concurrent writers.
    id: String,
    /// RFC 3339 is the most widely supported machine-readable timestamp format.
    ts: String,
    /// Severity filtering in queries needs the level as a queryable field.
    level: String,
    /// Queries often filter by subsystem — `scope` is the primary grouping dimension.
    scope: String,
    /// Stripped of XML-style tags — JSONL consumers expect clean text, not ANSI or markup.
    msg: String,
    /// Multi-app setups share one JSONL file — the app field lets queries filter by application.
    #[serde(skip_serializing_if = "Option::is_none")]
    app: Option<String>,
    /// Presets use domain-specific labels ("SUCCESS", "DEPLOY") — preserving them enables richer queries.
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
}

/// Append-only JSONL file — one JSON object per line creates a queryable log database
/// without the complexity of a real database engine.
#[derive(Debug, Clone)]
pub struct JsonOutput {
    /// Default XDG path doesn't work for every deployment.
    file_path: PathBuf,
    /// JSONL records need an app field so stats queries can filter by application.
    app_name: Option<String>,
}

impl Default for JsonOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonOutput {
    /// Sensible XDG default path lets the builder work without any configuration for common setups.
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

    /// Default XDG path doesn't work for every deployment (containers, custom setups).
    #[must_use]
    pub fn path(mut self, path: impl Into<PathBuf>) -> Self {
        self.file_path = path.into();
        self
    }

    /// Multi-app setups share one JSONL file — the app name distinguishes entries.
    #[must_use]
    pub fn app_name(mut self, name: impl Into<String>) -> Self {
        self.app_name = Some(name.into());
        self
    }

    /// Config values use `~` for portability — the OS needs an absolute path for file operations.
    fn resolve_path(&self) -> PathBuf {
        let path_str = self.file_path.to_string_lossy();
        let expanded = shellexpand::tilde(&path_str);
        let path = PathBuf::from(expanded.as_ref());
        internal::trace("JSON", &format!("Resolved path: {}", path.display()));
        path
    }

    /// Transforms the internal `LogRecord` into the flat JSONL schema — strips styling and generates a ULID.
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
