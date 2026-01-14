//! Inline message styling with XML-like tags.
//!
//! Supports tags like `<bold>text</bold>` and `<red>text</red>`.

use crate::Color;
use std::collections::HashMap;

/// A styled segment of text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment {
    /// Plain text without styling.
    Plain(String),
    /// Bold text.
    Bold(String),
    /// Dimmed text.
    Dim(String),
    /// Italic text.
    Italic(String),
    /// Underlined text.
    Underline(String),
    /// Colored text (named color or hex).
    Colored(String, String),
}

impl Segment {
    /// Returns the inner text without styling.
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

    /// Renders the segment with ANSI escape codes.
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

    /// Returns plain text without any ANSI codes.
    #[must_use]
    pub fn render_plain(&self) -> String {
        self.text().to_string()
    }
}

/// Parses a message string into styled segments.
///
/// Supports: `<bold>`, `<dim>`, `<italic>`, `<underline>`, and any color name.
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

/// Renders parsed segments to a styled string.
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn render(segments: &[Segment], colors: &HashMap<String, Color>) -> String {
    segments.iter().map(|s| s.render(colors)).collect()
}

/// Renders parsed segments to plain text (strips all styling).
#[must_use]
pub fn render_plain(segments: &[Segment]) -> String {
    segments.iter().map(Segment::render_plain).collect()
}

/// Strips all XML-like tags from a message, returning plain text.
#[must_use]
pub fn strip_tags(msg: &str) -> String {
    render_plain(&parse(msg))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_plain() {
        let segments = parse("hello world");
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0], Segment::Plain("hello world".to_string()));
    }

    #[test]
    fn parse_bold() {
        let segments = parse("hello <bold>world</bold>");
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0], Segment::Plain("hello ".to_string()));
        assert_eq!(segments[1], Segment::Bold("world".to_string()));
    }

    #[test]
    fn parse_colored() {
        let segments = parse("<red>error</red> message");
        assert_eq!(segments.len(), 2);
        assert_eq!(
            segments[0],
            Segment::Colored("error".to_string(), "red".to_string())
        );
        assert_eq!(segments[1], Segment::Plain(" message".to_string()));
    }

    #[test]
    fn parse_multiple() {
        let segments = parse("<bold>A</bold> and <italic>B</italic>");
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0], Segment::Bold("A".to_string()));
        assert_eq!(segments[1], Segment::Plain(" and ".to_string()));
        assert_eq!(segments[2], Segment::Italic("B".to_string()));
    }

    #[test]
    fn strip_tags_test() {
        let result = strip_tags("<bold>hello</bold> <red>world</red>");
        assert_eq!(result, "hello world");
    }

    #[test]
    fn render_bold() {
        let segments = vec![Segment::Bold("test".to_string())];
        let result = render(&segments, &HashMap::new());
        assert_eq!(result, "\x1b[1mtest\x1b[0m");
    }

    #[test]
    fn render_colored_hex() {
        let segments = vec![Segment::Colored("test".to_string(), "#ff0000".to_string())];
        let result = render(&segments, &HashMap::new());
        assert!(result.contains("38;2;255;0;0"));
    }
}
