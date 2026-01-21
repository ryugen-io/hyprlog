//! Scope formatting for log output.

use super::tag::{Alignment, Transform};

/// Configuration for scope formatting.
#[derive(Debug, Clone)]
pub struct ScopeConfig {
    /// Minimum width (padded if shorter).
    pub min_width: usize,
    /// Text alignment within `min_width`.
    pub alignment: Alignment,
    /// Text transformation.
    pub transform: Transform,
}

impl Default for ScopeConfig {
    fn default() -> Self {
        Self {
            min_width: 12,
            alignment: Alignment::Left,
            transform: Transform::None,
        }
    }
}

impl ScopeConfig {
    /// Creates a new scope config with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
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

    /// Sets the transform.
    #[must_use]
    pub const fn transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    /// Formats a scope string with padding and transformation.
    #[must_use]
    pub fn format(&self, scope: &str) -> String {
        let transformed = self.transform.apply(scope);
        self.pad(&transformed)
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
