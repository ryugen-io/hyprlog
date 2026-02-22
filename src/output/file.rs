//! Persistent logs survive process restarts — critical for post-mortem debugging
//! when terminal scrollback is lost or the session crashed.

use crate::fmt::{FormatTemplate, FormatValues, TagConfig, style};
use crate::internal;

use super::{LogRecord, Output};
use chrono::Local;
use std::cell::Cell;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

// Thread-local recursion guard to prevent deadlock when internal logging
// triggers file output which tries to log again.
thread_local! {
    static IN_FILE_WRITE: Cell<bool> = const { Cell::new(false) };
}

/// All file-output state in one struct — path templates, timestamp format, and a write buffer for batching raw items.
pub struct FileOutput {
    /// Default XDG path doesn't work for every deployment (containers, custom setups).
    base_dir: String,
    /// Different projects organize logs differently (by app, by date, flat).
    path_template: FormatTemplate,
    /// Multiple apps in the same directory need distinct filenames.
    filename_template: FormatTemplate,
    /// File output doesn't need ANSI but may need timestamps or different column order than terminal.
    content_template: FormatTemplate,
    /// Different locales and log analysis tools expect different timestamp formats.
    timestamp_format: String,
    /// Without this, all apps would dump logs into the same directory.
    app_name: String,
    /// File logs often need simpler tags than terminal for grep-friendliness.
    tag_config: TagConfig,
    /// Raw items (list entries) are collected and appended to the preceding header line on flush.
    buffer: Mutex<Option<BufferedLine>>,
}

/// Groups a header line with its following raw items so they're written as one logical entry.
struct BufferedLine {
    /// The timestamp + tag + scope + message content that starts the log entry.
    content: String,
    /// Resolved once at buffer creation — avoids re-expanding templates for every raw item.
    path: PathBuf,
    /// Raw `logger.raw()` calls append here until the next normal log line triggers a flush.
    items: Vec<String>,
}

impl Default for FileOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl FileOutput {
    /// Sensible XDG defaults let the builder work without any configuration for common setups.
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

    /// Default XDG path doesn't work for every deployment (containers, custom setups).
    #[must_use]
    pub fn base_dir(mut self, dir: impl Into<String>) -> Self {
        self.base_dir = dir.into();
        self
    }

    /// Different projects organize logs differently (by app, by date, flat, etc.).
    #[must_use]
    pub fn path_structure(mut self, template: &str) -> Self {
        self.path_template = FormatTemplate::parse(template);
        self
    }

    /// Multiple apps logging to the same directory need distinct filenames.
    #[must_use]
    pub fn filename_structure(mut self, template: &str) -> Self {
        self.filename_template = FormatTemplate::parse(template);
        self
    }

    /// File output doesn't need ANSI but may need timestamps or different column order.
    #[must_use]
    pub fn content_structure(mut self, template: &str) -> Self {
        self.content_template = FormatTemplate::parse(template);
        self
    }

    /// Different locales and log analysis tools expect different timestamp formats.
    #[must_use]
    pub fn timestamp_format(mut self, format: impl Into<String>) -> Self {
        self.timestamp_format = format.into();
        self
    }

    /// Without this, all apps would dump logs into the same directory.
    #[must_use]
    pub fn app_name(mut self, name: impl Into<String>) -> Self {
        self.app_name = name.into();
        self
    }

    /// File logs often need simpler tags than terminal for grep-friendliness.
    #[must_use]
    pub fn tag_config(mut self, config: TagConfig) -> Self {
        self.tag_config = config;
        self
    }

    /// Config values use `~` for portability — the OS needs an absolute path for `fs::create_dir_all`.
    fn resolve_base_dir(&self) -> PathBuf {
        let expanded = shellexpand::tilde(&self.base_dir);
        let path = PathBuf::from(expanded.as_ref());
        // Only log if not already inside a file write (prevents deadlock)
        if !IN_FILE_WRITE.with(Cell::get) {
            internal::trace("FILE", &format!("Resolved base dir: {}", path.display()));
        }
        path
    }

    /// Combines base dir + path template + filename template so each record lands in the right file.
    fn build_path(&self, record: &LogRecord) -> PathBuf {
        let base = self.resolve_base_dir();
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

        base.join(rel_path).join(filename)
    }

    /// Strips ANSI tags and applies the content template — file output must be plain text for grep/awk.
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
    /// Flushes the accumulated header + raw items as a single line to avoid interleaved writes from concurrent threads.
    fn write_buffered(buf: &BufferedLine) -> Result<(), crate::Error> {
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
            line.push(' ');
            line.push_str(&buf.items.join(", "));
        }
        line.push('\n');
        file.write_all(line.as_bytes())?;
        Ok(())
    }

    /// Separated from the `Output::write` impl so the recursion guard wraps this cleanly.
    fn write_inner(&self, record: &LogRecord) -> Result<(), crate::Error> {
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
        let path = self.build_path(record);

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
}

impl Output for FileOutput {
    fn write(&self, record: &LogRecord) -> Result<(), crate::Error> {
        // Set recursion guard to prevent deadlock from internal logging
        IN_FILE_WRITE.with(|flag| flag.set(true));
        let result = self.write_inner(record);
        IN_FILE_WRITE.with(|flag| flag.set(false));
        result
    }

    fn flush(&self) -> Result<(), crate::Error> {
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
