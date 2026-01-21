//! File output with path templates.

use crate::fmt::{FormatTemplate, FormatValues, TagConfig, style};
use crate::internal;

use super::{LogRecord, Output, OutputError};
use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

/// File output configuration.
pub struct FileOutput {
    /// Base directory for log files.
    base_dir: String,
    /// Path structure template (e.g., `{year}/{month}/{app}`).
    path_template: FormatTemplate,
    /// Filename structure template (e.g., `{level}_{day}.log`).
    filename_template: FormatTemplate,
    /// Content structure template.
    content_template: FormatTemplate,
    /// Timestamp format (strftime).
    timestamp_format: String,
    /// Application name for templates.
    app_name: String,
    /// Tag formatting config.
    tag_config: TagConfig,
    /// Buffered line (header + raw items collected).
    buffer: Mutex<Option<BufferedLine>>,
}

/// A buffered log line with collected raw items.
struct BufferedLine {
    /// The formatted header line.
    content: String,
    /// Path to write to.
    path: PathBuf,
    /// Collected raw items.
    items: Vec<String>,
}

impl Default for FileOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl FileOutput {
    /// Creates a new file output with defaults.
    #[must_use]
    pub fn new() -> Self {
        let base_dir = directories::ProjectDirs::from("", "", "hyprlog").map_or_else(
            || "logs".to_string(),
            |dirs| {
                dirs.state_dir()
                    .unwrap_or_else(|| dirs.data_dir())
                    .join("logs")
                    .to_string_lossy()
                    .into_owned()
            },
        );

        Self {
            base_dir,
            path_template: FormatTemplate::parse("{year}/{month}/{app}"),
            filename_template: FormatTemplate::parse("{scope}_{level}_{day}.log"),
            content_template: FormatTemplate::parse("{timestamp} {tag} {scope}  {msg}"),
            timestamp_format: "%Y-%m-%d %H:%M:%S".to_string(),
            app_name: "hyprlog".to_string(),
            tag_config: TagConfig::default(),
            buffer: Mutex::new(None),
        }
    }

    /// Sets the base directory.
    #[must_use]
    pub fn base_dir(mut self, dir: impl Into<String>) -> Self {
        self.base_dir = dir.into();
        self
    }

    /// Sets the path structure template.
    #[must_use]
    pub fn path_structure(mut self, template: &str) -> Self {
        self.path_template = FormatTemplate::parse(template);
        self
    }

    /// Sets the filename structure template.
    #[must_use]
    pub fn filename_structure(mut self, template: &str) -> Self {
        self.filename_template = FormatTemplate::parse(template);
        self
    }

    /// Sets the content structure template.
    #[must_use]
    pub fn content_structure(mut self, template: &str) -> Self {
        self.content_template = FormatTemplate::parse(template);
        self
    }

    /// Sets the timestamp format.
    #[must_use]
    pub fn timestamp_format(mut self, format: impl Into<String>) -> Self {
        self.timestamp_format = format.into();
        self
    }

    /// Sets the application name.
    #[must_use]
    pub fn app_name(mut self, name: impl Into<String>) -> Self {
        self.app_name = name.into();
        self
    }

    /// Sets the tag configuration.
    #[must_use]
    pub fn tag_config(mut self, config: TagConfig) -> Self {
        self.tag_config = config;
        self
    }

    /// Resolves the base directory (expands ~).
    fn resolve_base_dir(&self) -> Result<PathBuf, OutputError> {
        let path = if self.base_dir.starts_with('~') {
            if let Some(user_dirs) = directories::UserDirs::new() {
                if let Some(home) = user_dirs.home_dir().to_str() {
                    PathBuf::from(self.base_dir.replacen('~', home, 1))
                } else {
                    return Err(OutputError::Format(
                        "home directory path contains invalid utf-8".to_string(),
                    ));
                }
            } else {
                return Err(OutputError::Format(
                    "could not resolve home directory".to_string(),
                ));
            }
        } else {
            PathBuf::from(&self.base_dir)
        };
        internal::trace("FILE", &format!("Resolved base dir: {}", path.display()));
        Ok(path)
    }

    /// Builds the full file path for a record.
    fn build_path(&self, record: &LogRecord) -> Result<PathBuf, OutputError> {
        let base = self.resolve_base_dir()?;
        let now = Local::now();

        let values = FormatValues::new()
            .level(record.level.as_str())
            .scope(&record.scope)
            .app(record.app_name.as_deref().unwrap_or(&self.app_name))
            .date(
                &now.format("%Y").to_string(),
                &now.format("%m").to_string(),
                &now.format("%d").to_string(),
            );

        let rel_path = self.path_template.render(&values);
        let filename = self.filename_template.render(&values);

        Ok(base.join(rel_path).join(filename))
    }

    /// Formats the content line.
    fn format_content(&self, record: &LogRecord) -> String {
        let now = Local::now();
        let timestamp = now.format(&self.timestamp_format).to_string();
        let tag = record.format_tag(&self.tag_config);

        // Strip styling tags from message for file output
        let clean_msg = style::strip_tags(&record.message);

        let values = FormatValues::new()
            .timestamp(&timestamp)
            .tag(&tag)
            .scope(&record.scope)
            .msg(&clean_msg)
            .level(record.level.as_str())
            .app(record.app_name.as_deref().unwrap_or(&self.app_name));

        self.content_template.render(&values)
    }
}

impl FileOutput {
    /// Writes a buffered line to file.
    fn write_buffered(buf: &BufferedLine) -> Result<(), OutputError> {
        // Create directories
        if let Some(parent) = buf.path.parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&buf.path)?;

        // Build single line: header + items joined
        let mut line = buf.content.clone();
        if !buf.items.is_empty() {
            line.push_str(&buf.items.join(", "));
        }
        line.push('\n');
        file.write_all(line.as_bytes())?;
        Ok(())
    }
}

impl Output for FileOutput {
    fn write(&self, record: &LogRecord) -> Result<(), OutputError> {
        let mut buffer = self.buffer.lock().unwrap();

        if record.raw {
            // Raw message: collect into buffer items
            let clean = style::strip_tags(&record.message).trim().to_string();
            if let Some(ref mut buf) = *buffer {
                buf.items.push(clean);
            }
            // If no buffer exists, raw message is orphaned - ignore it
            return Ok(());
        }

        // Normal message: flush existing buffer first
        if let Some(ref buf) = *buffer {
            Self::write_buffered(buf)?;
        }

        // Build new buffered line
        let path = self.build_path(record)?;

        // Create directories if needed
        if let Some(parent) = path.parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent)?;
            internal::debug("FILE", &format!("Created directory: {}", parent.display()));
        }

        let content = self.format_content(record);

        *buffer = Some(BufferedLine {
            content,
            path,
            items: Vec::new(),
        });
        drop(buffer);

        Ok(())
    }

    fn flush(&self) -> Result<(), OutputError> {
        let mut buffer = self.buffer.lock().unwrap();
        if let Some(ref buf) = *buffer {
            Self::write_buffered(buf)?;
        }
        *buffer = None;
        drop(buffer);
        Ok(())
    }
}

impl Drop for FileOutput {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}
