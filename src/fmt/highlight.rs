//! Auto-highlighting for message content.
//!
//! Injects XML-style color tags for keywords and patterns before style parsing.

use crate::config::HighlightConfig;
use regex::Regex;
use std::sync::LazyLock;

/// Regex pattern for URLs (https://... or http://...).
static URL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"https?://[^\s<>]+").expect("Invalid URL regex"));

/// Regex pattern for file paths (/path, ~/path, ./path).
static PATH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:^|[^<\w])((?:/|~/|\./)[\w./-]+)").expect("Invalid path regex")
});

/// Regex pattern for quoted strings ("..." or '...').
static QUOTED_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#""[^"]*"|'[^']*'"#).expect("Invalid quoted regex"));

/// Regex pattern for numbers (integers and decimals).
static NUMBER_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b-?\d+(?:\.\d+)?\b").expect("Invalid number regex"));

/// Regex pattern for existing XML-style tags.
static EXISTING_TAG_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<[^>]+>[^<]*</[^>]+>").expect("Invalid tag regex"));

/// A span representing a region in the text.
#[derive(Debug, Clone, Copy)]
struct Span {
    start: usize,
    end: usize,
}

impl Span {
    const fn overlaps(&self, other: &Self) -> bool {
        self.start < other.end && other.start < self.end
    }
}

/// A match with its span and color.
#[derive(Debug)]
struct Match {
    span: Span,
    text: String,
    color: String,
}

/// Injects XML-style color tags into a message for auto-highlighting.
///
/// This function identifies keywords and patterns (URLs, paths, numbers, quoted strings)
/// and wraps them in color tags before the style parser processes them.
///
/// # Arguments
/// * `msg` - The message to process.
/// * `config` - Highlight configuration with keywords and patterns.
///
/// # Returns
/// The message with injected color tags.
pub fn inject_tags(msg: &str, config: &HighlightConfig) -> String {
    if !config.enabled || msg.is_empty() {
        return msg.to_string();
    }

    // Find existing tags to skip
    let existing_spans: Vec<Span> = EXISTING_TAG_REGEX
        .find_iter(msg)
        .map(|m| Span {
            start: m.start(),
            end: m.end(),
        })
        .collect();

    let mut matches: Vec<Match> = Vec::new();

    // Match patterns in priority order: URLs > Paths > Quoted > Numbers
    if let Some(color) = &config.patterns.urls {
        for m in URL_REGEX.find_iter(msg) {
            let span = Span {
                start: m.start(),
                end: m.end(),
            };
            if !overlaps_any(&span, &existing_spans) {
                matches.push(Match {
                    span,
                    text: m.as_str().to_string(),
                    color: color.clone(),
                });
            }
        }
    }

    if let Some(color) = &config.patterns.paths {
        for cap in PATH_REGEX.captures_iter(msg) {
            if let Some(m) = cap.get(1) {
                let span = Span {
                    start: m.start(),
                    end: m.end(),
                };
                if !overlaps_any(&span, &existing_spans) && !overlaps_any_match(&span, &matches) {
                    matches.push(Match {
                        span,
                        text: m.as_str().to_string(),
                        color: color.clone(),
                    });
                }
            }
        }
    }

    if let Some(color) = &config.patterns.quoted {
        for m in QUOTED_REGEX.find_iter(msg) {
            let span = Span {
                start: m.start(),
                end: m.end(),
            };
            if !overlaps_any(&span, &existing_spans) && !overlaps_any_match(&span, &matches) {
                matches.push(Match {
                    span,
                    text: m.as_str().to_string(),
                    color: color.clone(),
                });
            }
        }
    }

    if let Some(color) = &config.patterns.numbers {
        for m in NUMBER_REGEX.find_iter(msg) {
            let span = Span {
                start: m.start(),
                end: m.end(),
            };
            if !overlaps_any(&span, &existing_spans) && !overlaps_any_match(&span, &matches) {
                matches.push(Match {
                    span,
                    text: m.as_str().to_string(),
                    color: color.clone(),
                });
            }
        }
    }

    // Match keywords (case-insensitive word boundaries)
    for (keyword, color) in &config.keywords {
        let pattern = format!(r"(?i)\b{}\b", regex::escape(keyword));
        if let Ok(re) = Regex::new(&pattern) {
            for m in re.find_iter(msg) {
                let span = Span {
                    start: m.start(),
                    end: m.end(),
                };
                if !overlaps_any(&span, &existing_spans) && !overlaps_any_match(&span, &matches) {
                    matches.push(Match {
                        span,
                        text: m.as_str().to_string(),
                        color: color.clone(),
                    });
                }
            }
        }
    }

    // Sort matches by position (reverse order for replacement)
    matches.sort_by(|a, b| b.span.start.cmp(&a.span.start));

    // Build result by replacing matches
    let mut result = msg.to_string();
    for m in matches {
        let replacement = format!("<{}>{}</{}>", m.color, m.text, m.color);
        result.replace_range(m.span.start..m.span.end, &replacement);
    }

    result
}

/// Checks if a span overlaps with any span in the list.
fn overlaps_any(span: &Span, spans: &[Span]) -> bool {
    spans.iter().any(|s| span.overlaps(s))
}

/// Checks if a span overlaps with any match in the list.
fn overlaps_any_match(span: &Span, matches: &[Match]) -> bool {
    matches.iter().any(|m| span.overlaps(&m.span))
}
