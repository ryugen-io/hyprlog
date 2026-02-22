//! Every project has a different convention for level indicators (`[INFO]` vs `<info>` vs `INFO:`)
//! — a configurable tag system avoids hardcoding any single style.

use crate::level::Level;
use std::collections::HashMap;

/// Hyprland uses lowercase level names, most loggers use uppercase — users need to match their convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Transform {
    /// Some projects already control casing upstream — double-transforming would mangle them.
    #[default]
    None,
    /// Most log conventions expect `INFO`/`WARN`/`ERROR` in all-caps for quick scanning.
    Uppercase,
    /// Hyprland's native log format uses lowercase — matching it avoids visual mismatch.
    Lowercase,
    /// Title-case (`Info`, `Warn`) looks cleaner in prose-style log formats.
    Capitalize,
}

impl Transform {
    /// Centralized transform avoids duplicating casing logic at every call site.
    #[must_use]
    pub fn apply(self, s: &str) -> String {
        match self {
            Self::None => s.to_string(),
            Self::Uppercase => s.to_uppercase(),
            Self::Lowercase => s.to_lowercase(),
            Self::Capitalize => {
                let mut chars = s.chars();
                chars.next().map_or_else(String::new, |first| {
                    first.to_uppercase().collect::<String>()
                        + chars.as_str().to_lowercase().as_str()
                })
            }
        }
    }
}

/// Centered tags look cleaner with padding; left-aligned are more grep-friendly — users need the choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Alignment {
    /// Grep and `cut` expect fixed-offset columns — left alignment keeps the tag start predictable.
    Left,
    /// Right-aligned tags keep the message column start consistent regardless of tag length.
    Right,
    /// Centered text looks balanced inside padded brackets — the most common visual preference.
    #[default]
    Center,
}

/// Every knob in one struct so tag rendering doesn't need to accept a dozen loose parameters.
#[derive(Debug, Clone)]
pub struct TagConfig {
    /// Opening delimiter — `[` produces `[INFO]`, `<` produces `<INFO>`, empty gives bare `INFO`.
    pub prefix: String,
    /// Closing delimiter — must pair with prefix for readability (`[INFO]` not `[INFO>`).
    pub suffix: String,
    /// Casing convention varies across ecosystems — the tag must match the project's style.
    pub transform: Transform,
    /// Level names have different lengths — padding keeps columns aligned across `INFO`/`WARN`/`ERROR`.
    pub min_width: usize,
    /// Different alignment choices affect readability vs. machine-parseability.
    pub alignment: Alignment,
    /// Projects may want domain-specific names instead of "INFO"/"WARN" (e.g., "OK", "FAIL").
    pub labels: HashMap<Level, String>,
}

impl Default for TagConfig {
    fn default() -> Self {
        Self {
            prefix: "[".to_string(),
            suffix: "]".to_string(),
            transform: Transform::Uppercase,
            min_width: 5,
            alignment: Alignment::Center,
            labels: HashMap::new(),
        }
    }
}

impl TagConfig {
    /// Explicit constructor matches the builder-pattern convention used throughout the crate.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Different log formats use different opening delimiters (`[`, `<`, or none).
    #[must_use]
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// Closing delimiter must be independently configurable to pair with any opening delimiter.
    #[must_use]
    pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = suffix.into();
        self
    }

    /// Casing preference varies across projects — uppercase for traditional logs, lowercase for Hyprland-style.
    #[must_use]
    pub const fn transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    /// Without minimum width, `[INFO]` and `[WARN]` produce different column offsets.
    #[must_use]
    pub const fn min_width(mut self, width: usize) -> Self {
        self.min_width = width;
        self
    }

    /// Alignment within the padded width affects whether tags are human-scannable or machine-parseable.
    #[must_use]
    pub const fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Domain-specific names ("OK", "FAIL", "SKIP") communicate intent better than generic level names.
    #[must_use]
    pub fn label(mut self, level: Level, label: impl Into<String>) -> Self {
        self.labels.insert(level, label.into());
        self
    }

    /// Single entry point for tag rendering — applies transform, padding, and delimiters in the correct order.
    #[must_use]
    pub fn format(&self, level: Level) -> String {
        let label = self
            .labels
            .get(&level)
            .map_or_else(|| level.as_str(), String::as_str);

        let transformed = self.transform.apply(label);
        let padded = self.pad(&transformed);

        format!("{}{}{}", self.prefix, padded, self.suffix)
    }

    /// Presets and custom events need arbitrary labels that don't map to any built-in level name.
    #[must_use]
    pub fn format_with_label(&self, _level: Level, label: &str) -> String {
        let transformed = self.transform.apply(label);
        let padded = self.pad(&transformed);
        format!("{}{}{}", self.prefix, padded, self.suffix)
    }

    fn pad(&self, s: &str) -> String {
        let len = s.chars().count();
        if len >= self.min_width {
            return s.to_string();
        }

        let padding = self.min_width - len;
        match self.alignment {
            Alignment::Left => format!("{}{}", s, " ".repeat(padding)),
            Alignment::Right => format!("{}{}", " ".repeat(padding), s),
            Alignment::Center => {
                let left = padding / 2;
                let right = padding - left;
                format!("{}{}{}", " ".repeat(left), s, " ".repeat(right))
            }
        }
    }
}
