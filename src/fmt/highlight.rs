//! Dense log output buries URLs, paths, and numbers in a wall of text.
//!
//! Auto-highlighting injects color tags before the style parser runs so these tokens
//! stand out without the user manually wrapping every occurrence.

use crate::config::HighlightConfig;
use regex::Regex;
use std::sync::LazyLock;

/// URLs in logs are often clickable in modern terminals — highlighting them makes them findable.
static URL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"https?://[^\s<>]+").expect("Invalid URL regex"));

/// File paths in error messages are the first thing users look for when diagnosing failures.
static PATH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:^|[^<\w])((?:/|~/|\./)[\w./-]+)").expect("Invalid path regex")
});

/// Quoted strings often contain user input or error messages worth distinguishing from surrounding text.
static QUOTED_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#""[^"]*"|'[^']*'"#).expect("Invalid quoted regex"));

/// Numeric values (ports, counts, durations) are key diagnostic data that blends into prose without color.
static NUMBER_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b-?\d+(?:\.\d+)?\b").expect("Invalid number regex"));

/// Already-tagged regions must be skipped — double-wrapping would produce nested `<color>` tags that break rendering.
static EXISTING_TAG_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<[^>]+>[^<]*</[^>]+>").expect("Invalid tag regex"));

/// Overlap detection needs start/end pairs — a tuple would lose semantic clarity.
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

/// Each regex hit needs its position, original text, and target color for the replacement pass.
#[derive(Debug)]
struct Match {
    span: Span,
    text: String,
    color: String,
}

/// Runs before the style parser — wraps URLs, paths, numbers, and keywords in `<color>` tags
/// so the existing style pipeline renders them without special-case logic.
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

/// Prevents double-tagging regions that are already inside an existing XML tag.
fn overlaps_any(span: &Span, spans: &[Span]) -> bool {
    spans.iter().any(|s| span.overlaps(s))
}

/// Lower-priority patterns (numbers) must not re-tag text already claimed by higher-priority patterns (URLs).
fn overlaps_any_match(span: &Span, matches: &[Match]) -> bool {
    matches.iter().any(|m| span.overlaps(&m.span))
}
