//! 256-color palettes can't match Hyprland themes like Catppuccin or Dracula —
//! 24-bit true color is the minimum fidelity for accurate gradient rendering.

use std::fmt;

/// A dedicated type prevents mixing up raw u8 triples and documents color intent at the type level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// `const` so theme palettes and named colors can be compile-time constants.
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Config files specify colors as `#RRGGBB` hex strings — this converts them
    /// to the numeric triple the ANSI escape builder needs. Falls back to white
    /// on malformed input so a typo in config doesn't crash rendering.
    #[must_use]
    pub fn from_hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 || !hex.is_ascii() {
            return Self::white();
        }

        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);

        Self { r, g, b }
    }

    /// Terminals need the raw `\x1b[38;2;R;G;Bm` escape — callers shouldn't hand-build it.
    #[must_use]
    pub fn fg_ansi(self) -> String {
        format!("\x1b[38;2;{};{};{}m", self.r, self.g, self.b)
    }

    /// Background coloring uses a different SGR code (48 vs 38) — exposing a separate method avoids caller confusion.
    #[must_use]
    pub fn bg_ansi(self) -> String {
        format!("\x1b[48;2;{};{};{}m", self.r, self.g, self.b)
    }

    /// Terminates any active SGR styling so subsequent text returns to the terminal default.
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

    #[must_use]
    pub const fn blue() -> Self {
        Self::new(98, 114, 164)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

/// Convenience wrapper — most callers just want "make this text colored" without managing reset sequences.
#[must_use]
pub fn colorize(text: &str, color: Color) -> String {
    let fg = color.fg_ansi();
    let reset = Color::RESET;
    format!("{fg}{text}{reset}")
}

/// Badge-style rendering (colored background + contrasting text) needs both FG and BG escapes paired together.
#[must_use]
pub fn colorize_bg(text: &str, fg: Color, bg: Color) -> String {
    let fg_code = fg.fg_ansi();
    let bg_code = bg.bg_ansi();
    let reset = Color::RESET;
    format!("{fg_code}{bg_code}{text}{reset}")
}
