//! Tag formatting for log levels.

use crate::Level;
use std::collections::HashMap;

/// Text transformation for tags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Transform {
    /// No transformation.
    #[default]
    None,
    /// UPPERCASE.
    Uppercase,
    /// lowercase.
    Lowercase,
    /// Capitalize first letter.
    Capitalize,
}

impl Transform {
    /// Applies the transformation to a string.
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

/// Text alignment for tags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Alignment {
    /// Left-aligned.
    Left,
    /// Right-aligned.
    Right,
    /// Center-aligned.
    #[default]
    Center,
}

/// Configuration for tag formatting.
#[derive(Debug, Clone)]
pub struct TagConfig {
    /// Prefix before the tag (e.g. `[`).
    pub prefix: String,
    /// Suffix after the tag (e.g. `]`).
    pub suffix: String,
    /// Text transformation.
    pub transform: Transform,
    /// Minimum width (padded if shorter).
    pub min_width: usize,
    /// Text alignment within `min_width`.
    pub alignment: Alignment,
    /// Custom labels per level (overrides default level name).
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
    /// Creates a new tag config with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the prefix.
    #[must_use]
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// Sets the suffix.
    #[must_use]
    pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = suffix.into();
        self
    }

    /// Sets the transform.
    #[must_use]
    pub const fn transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    /// Sets the minimum width.
    #[must_use]
    pub const fn min_width(mut self, width: usize) -> Self {
        self.min_width = width;
        self
    }

    /// Sets the alignment.
    #[must_use]
    pub const fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Sets a custom label for a level.
    #[must_use]
    pub fn label(mut self, level: Level, label: impl Into<String>) -> Self {
        self.labels.insert(level, label.into());
        self
    }

    /// Formats a tag for the given level.
    #[must_use]
    pub fn format(&self, level: Level) -> String {
        // Get label (custom or default)
        let label = self
            .labels
            .get(&level)
            .map_or_else(|| level.as_str(), String::as_str);

        // Apply transformation
        let transformed = self.transform.apply(label);

        // Apply padding
        let padded = self.pad(&transformed);

        // Apply brackets
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transform_apply() {
        assert_eq!(Transform::None.apply("Test"), "Test");
        assert_eq!(Transform::Uppercase.apply("test"), "TEST");
        assert_eq!(Transform::Lowercase.apply("TEST"), "test");
        assert_eq!(Transform::Capitalize.apply("tEST"), "Test");
    }

    #[test]
    fn default_format() {
        let config = TagConfig::default();
        assert_eq!(config.format(Level::Info), "[INFO ]");
        assert_eq!(config.format(Level::Error), "[ERROR]");
    }

    #[test]
    fn custom_label() {
        let config = TagConfig::new().label(Level::Error, "FAIL");
        assert_eq!(config.format(Level::Error), "[FAIL ]");
    }

    #[test]
    fn no_brackets() {
        let config = TagConfig::new().prefix("").suffix("");
        assert_eq!(config.format(Level::Info), "INFO ");
    }

    #[test]
    fn left_alignment() {
        let config = TagConfig::new().alignment(Alignment::Left).min_width(8);
        assert_eq!(config.format(Level::Info), "[INFO    ]");
    }

    #[test]
    fn right_alignment() {
        let config = TagConfig::new().alignment(Alignment::Right).min_width(8);
        assert_eq!(config.format(Level::Info), "[    INFO]");
    }
}
