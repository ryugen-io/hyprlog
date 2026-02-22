//! Log messages sometimes need emphasis on specific words — XML-like tags (`<bold>`, `<red>`)
//! let users embed styling intent without coupling to ANSI escape codes directly.

use super::Color;
use std::collections::HashMap;

/// Parsed segments separate content from style so the same message can render with ANSI or as plain text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment {
    /// Text outside any tag needs no ANSI wrapping — rendering it as-is avoids unnecessary escapes.
    Plain(String),
    /// Bold draws the eye to critical keywords in a log line.
    Bold(String),
    /// Dim de-emphasizes low-priority metadata so it doesn't compete with the message.
    Dim(String),
    /// Italic distinguishes quoted or secondary information from the primary message.
    Italic(String),
    /// Underline marks actionable items (URLs, paths) that a user might click or copy.
    Underline(String),
    /// Color tags reference named theme colors or raw hex — decouples styling from ANSI codes.
    Colored(String, String),
}

impl Segment {
    /// File output and width calculations need the raw text without ANSI escapes inflating the length.
    #[must_use]
    pub fn text(&self) -> &str {
        match self {
            Self::Plain(t)
            | Self::Bold(t)
            | Self::Dim(t)
            | Self::Italic(t)
            | Self::Underline(t)
            | Self::Colored(t, _) => t,
        }
    }

    /// Terminal output needs ANSI wrapping — color names are resolved against the active theme map.
    #[must_use]
    pub fn render(&self, colors: &HashMap<String, Color>) -> String {
        match self {
            Self::Plain(t) => t.clone(),
            Self::Bold(t) => format!("\x1b[1m{t}\x1b[0m"),
            Self::Dim(t) => format!("\x1b[2m{t}\x1b[0m"),
            Self::Italic(t) => format!("\x1b[3m{t}\x1b[0m"),
            Self::Underline(t) => format!("\x1b[4m{t}\x1b[0m"),
            Self::Colored(t, name) => {
                let color = if name.starts_with('#') {
                    Color::from_hex(name)
                } else {
                    colors.get(name).copied().unwrap_or(Color::white())
                };
                let fg = color.fg_ansi();
                format!("{fg}{t}\x1b[0m")
            }
        }
    }

    /// File and JSON outputs must strip ANSI — they need the text content only.
    #[must_use]
    pub fn render_plain(&self) -> String {
        self.text().to_string()
    }
}

/// Splitting the message into typed segments before rendering lets the same parse result
/// serve both ANSI-capable and plain-text outputs without re-parsing.
#[must_use]
pub fn parse(msg: &str) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut i = 0;
    let bytes = msg.as_bytes();

    while i < bytes.len() {
        if bytes[i] == b'<' {
            // Look for closing >
            if let Some(tag_end) = find_char(bytes, i + 1, b'>') {
                let tag_name = &msg[i + 1..tag_end];

                // Skip closing tags
                if tag_name.starts_with('/') {
                    i = tag_end + 1;
                    continue;
                }

                // Find matching close tag
                let close_tag = format!("</{tag_name}>");
                if let Some(content_end) = msg[tag_end + 1..].find(&close_tag) {
                    let content_start = tag_end + 1;
                    let content_end = content_start + content_end;
                    let content = &msg[content_start..content_end];

                    let segment = match tag_name.to_lowercase().as_str() {
                        "bold" | "b" => Segment::Bold(content.to_string()),
                        "dim" => Segment::Dim(content.to_string()),
                        "italic" | "i" => Segment::Italic(content.to_string()),
                        "underline" | "u" => Segment::Underline(content.to_string()),
                        _ => Segment::Colored(content.to_string(), tag_name.to_string()),
                    };

                    segments.push(segment);
                    i = content_end + close_tag.len();
                    continue;
                }
            }
        }

        // Find next < or end of string
        let next_tag = find_char(bytes, i, b'<').unwrap_or(bytes.len());
        if next_tag > i {
            segments.push(Segment::Plain(msg[i..next_tag].to_string()));
        }
        i = next_tag;
    }

    segments
}

fn find_char(bytes: &[u8], start: usize, c: u8) -> Option<usize> {
    bytes[start..]
        .iter()
        .position(|&b| b == c)
        .map(|p| start + p)
}

/// Terminal output needs the full ANSI rendering — this is the styled counterpart of `render_plain`.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn render(segments: &[Segment], colors: &HashMap<String, Color>) -> String {
    segments.iter().map(|s| s.render(colors)).collect()
}

/// File output and JSON need clean text — ANSI escapes would corrupt structured data.
#[must_use]
pub fn render_plain(segments: &[Segment]) -> String {
    segments.iter().map(Segment::render_plain).collect()
}

/// Convenience shortcut — callers who only need plain text shouldn't have to manage intermediate segments.
#[must_use]
pub fn strip_tags(msg: &str) -> String {
    render_plain(&parse(msg))
}
