//! Scope names like "hyprland", "config", "cleanup" have wildly different lengths —
//! without padding and alignment, the message column jumps around and becomes unreadable.

use super::tag::{Alignment, Transform};

/// All scope-rendering knobs in one struct so formatting doesn't need a dozen loose parameters.
#[derive(Debug, Clone)]
pub struct ScopeConfig {
    /// Scopes have different lengths — padding keeps the message column aligned.
    pub min_width: usize,
    /// Left-aligned scopes are easiest to scan in most terminals.
    pub alignment: Alignment,
    /// Projects may prefer uppercase scopes for visual distinction from the message body.
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
    /// Explicit constructor matches the builder-pattern convention used throughout the crate.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Different projects have different scope name lengths — the padding target must be adjustable.
    #[must_use]
    pub const fn min_width(mut self, width: usize) -> Self {
        self.min_width = width;
        self
    }

    /// Alignment within the padded width affects readability vs. machine-parseability.
    #[must_use]
    pub const fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Casing preference varies across projects — uppercase scopes stand out, lowercase blend in.
    #[must_use]
    pub const fn transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    /// Single entry point for scope rendering — applies transform and padding in the correct order.
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
