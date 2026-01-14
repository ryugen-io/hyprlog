//! Color handling for terminal output.

use std::fmt;

/// RGB color for 24-bit true color terminal output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// Creates a new color from RGB values.
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Creates a color from a hex string (with or without `#` prefix).
    ///
    /// Returns white (`#ffffff`) for invalid input.
    #[must_use]
    pub fn from_hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return Self::white();
        }

        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);

        Self { r, g, b }
    }

    /// Returns the ANSI escape sequence for foreground color.
    #[must_use]
    pub fn fg_ansi(self) -> String {
        format!("\x1b[38;2;{};{};{}m", self.r, self.g, self.b)
    }

    /// Returns the ANSI escape sequence for background color.
    #[must_use]
    pub fn bg_ansi(self) -> String {
        format!("\x1b[48;2;{};{};{}m", self.r, self.g, self.b)
    }

    /// ANSI reset sequence.
    pub const RESET: &'static str = "\x1b[0m";

    // Sweet Dracula palette defaults
    #[must_use]
    pub const fn white() -> Self {
        Self::new(255, 255, 255)
    }

    #[must_use]
    pub const fn green() -> Self {
        Self::new(80, 250, 123)
    }

    #[must_use]
    pub const fn yellow() -> Self {
        Self::new(241, 250, 140)
    }

    #[must_use]
    pub const fn cyan() -> Self {
        Self::new(139, 233, 253)
    }

    #[must_use]
    pub const fn red() -> Self {
        Self::new(255, 85, 85)
    }

    #[must_use]
    pub const fn purple() -> Self {
        Self::new(189, 147, 249)
    }

    #[must_use]
    pub const fn pink() -> Self {
        Self::new(255, 121, 198)
    }

    #[must_use]
    pub const fn orange() -> Self {
        Self::new(255, 184, 108)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

/// Colorize a string with foreground color.
#[must_use]
pub fn colorize(text: &str, color: Color) -> String {
    let fg = color.fg_ansi();
    let reset = Color::RESET;
    format!("{fg}{text}{reset}")
}

/// Colorize a string with foreground and background colors.
#[must_use]
pub fn colorize_bg(text: &str, fg: Color, bg: Color) -> String {
    let fg_code = fg.fg_ansi();
    let bg_code = bg.bg_ansi();
    let reset = Color::RESET;
    format!("{fg_code}{bg_code}{text}{reset}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_hex_valid() {
        let c = Color::from_hex("#ff0000");
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 0);

        let c = Color::from_hex("00ff00");
        assert_eq!(c.r, 0);
        assert_eq!(c.g, 255);
        assert_eq!(c.b, 0);
    }

    #[test]
    fn from_hex_invalid() {
        let c = Color::from_hex("invalid");
        assert_eq!(c, Color::white());

        let c = Color::from_hex("#fff");
        assert_eq!(c, Color::white());
    }

    #[test]
    fn display() {
        let c = Color::new(255, 128, 0);
        assert_eq!(c.to_string(), "#ff8000");
    }

    #[test]
    fn fg_ansi_sequence() {
        let c = Color::new(80, 250, 123);
        assert_eq!(c.fg_ansi(), "\x1b[38;2;80;250;123m");
    }

    #[test]
    fn palette_colors() {
        assert_eq!(Color::green(), Color::new(80, 250, 123));
        assert_eq!(Color::red(), Color::new(255, 85, 85));
    }
}
