//! Icon sets for log output.

use crate::Level;
use std::collections::HashMap;

/// Icon set type selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IconType {
    /// Nerd Font icons (requires compatible font).
    #[default]
    NerdFont,
    /// ASCII-only icons (universal compatibility).
    Ascii,
    /// No icons.
    None,
}

/// Collection of icons for each log level.
#[derive(Debug, Clone)]
pub struct IconSet {
    icons: HashMap<Level, String>,
    icon_type: IconType,
}

impl IconSet {
    /// Creates a new icon set with Nerd Font icons.
    #[must_use]
    pub fn nerdfont() -> Self {
        let mut icons = HashMap::new();
        icons.insert(Level::Trace, "\u{f188}".to_string()); //
        icons.insert(Level::Debug, "\u{f188}".to_string()); //
        icons.insert(Level::Info, "\u{f05a}".to_string()); //
        icons.insert(Level::Warn, "\u{f071}".to_string()); //
        icons.insert(Level::Error, "\u{f057}".to_string()); //

        Self {
            icons,
            icon_type: IconType::NerdFont,
        }
    }

    /// Creates a new icon set with ASCII icons.
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

    /// Creates an empty icon set (no icons).
    #[must_use]
    pub fn none() -> Self {
        Self {
            icons: HashMap::new(),
            icon_type: IconType::None,
        }
    }

    /// Gets the icon for a log level.
    #[must_use]
    pub fn get(&self, level: Level) -> &str {
        self.icons.get(&level).map_or("", String::as_str)
    }

    /// Sets a custom icon for a level.
    pub fn set(&mut self, level: Level, icon: impl Into<String>) {
        self.icons.insert(level, icon.into());
    }

    /// Returns the icon type.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nerdfont_icons() {
        let icons = IconSet::nerdfont();
        assert!(!icons.get(Level::Info).is_empty());
        assert_eq!(icons.icon_type(), IconType::NerdFont);
    }

    #[test]
    fn ascii_icons() {
        let icons = IconSet::ascii();
        assert_eq!(icons.get(Level::Info), "[i]");
        assert_eq!(icons.get(Level::Error), "[x]");
        assert_eq!(icons.icon_type(), IconType::Ascii);
    }

    #[test]
    fn none_icons() {
        let icons = IconSet::none();
        assert_eq!(icons.get(Level::Info), "");
        assert_eq!(icons.icon_type(), IconType::None);
    }

    #[test]
    fn custom_icon() {
        let mut icons = IconSet::ascii();
        icons.set(Level::Info, "INFO");
        assert_eq!(icons.get(Level::Info), "INFO");
    }
}
