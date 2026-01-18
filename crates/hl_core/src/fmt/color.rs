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
