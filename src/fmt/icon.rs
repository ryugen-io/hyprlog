//! Not every terminal has `NerdFont` glyphs — a fallback chain (`NerdFont` → ASCII → none)
//! ensures log output renders correctly regardless of font support.

use crate::level::Level;
use std::collections::HashMap;

/// Font availability varies across environments — the user must choose which glyph family works for them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IconType {
    /// Most Hyprland users already have `NerdFont` — these glyphs give the richest visual feedback.
    #[default]
    NerdFont,
    /// SSH sessions, CI runners, and minimal containers can't render `NerdFont` — ASCII works everywhere.
    Ascii,
    /// Machine-parsed output and piped pipelines break if icons inject unexpected characters.
    None,
}

/// Bundles the glyph map with its type so downstream code can query icons without knowing which family is active.
#[derive(Debug, Clone)]
pub struct IconSet {
    icons: HashMap<Level, String>,
    icon_type: IconType,
}

impl IconSet {
    /// `NerdFont` is the most common icon font in Hyprland setups — sensible default for most users.
    #[must_use]
    pub fn nerdfont() -> Self {
        let mut icons = HashMap::new();
        icons.insert(Level::Trace, "\u{f188}".to_string());
        icons.insert(Level::Debug, "\u{f188}".to_string());
        icons.insert(Level::Info, "\u{f05a}".to_string());
        icons.insert(Level::Warn, "\u{f071}".to_string());
        icons.insert(Level::Error, "\u{f057}".to_string());

        Self {
            icons,
            icon_type: IconType::NerdFont,
        }
    }

    /// Fallback for environments where `NerdFont` isn't available — `[i]`/`[!]`/`[x]` still convey severity.
    #[must_use]
    pub fn ascii() -> Self {
        let mut icons = HashMap::new();
        icons.insert(Level::Trace, "[~]".to_string());
        icons.insert(Level::Debug, "[.]".to_string());
        icons.insert(Level::Info, "[i]".to_string());
        icons.insert(Level::Warn, "[!]".to_string());
        icons.insert(Level::Error, "[x]".to_string());

        Self {
            icons,
            icon_type: IconType::Ascii,
        }
    }

    /// Some outputs (file, JSON, piped) should never contain icon characters.
    #[must_use]
    pub fn none() -> Self {
        Self {
            icons: HashMap::new(),
            icon_type: IconType::None,
        }
    }

    /// Callers shouldn't need to know which icon family is active — they just need the glyph for a level.
    #[must_use]
    pub fn get(&self, level: Level) -> &str {
        self.icons.get(&level).map_or("", String::as_str)
    }

    /// Config-defined icon overrides let users replace built-in glyphs with their own preference.
    pub fn set(&mut self, level: Level, icon: impl Into<String>) {
        self.icons.insert(level, icon.into());
    }

    /// Downstream code may need to know the active family for conditional rendering decisions.
    #[must_use]
    pub const fn icon_type(&self) -> IconType {
        self.icon_type
    }
}

impl Default for IconSet {
    fn default() -> Self {
        Self::nerdfont()
    }
}

impl From<IconType> for IconSet {
    fn from(icon_type: IconType) -> Self {
        match icon_type {
            IconType::NerdFont => Self::nerdfont(),
            IconType::Ascii => Self::ascii(),
            IconType::None => Self::none(),
        }
    }
}
