use hyprlog::fmt::{Color, colorize, colorize_bg};

#[test]
fn from_hex_parses_valid_colors() {
    let color = Color::from_hex("#ff00aa");
    assert_eq!(color, Color::new(255, 0, 170));

    let color = Color::from_hex("01a2ff");
    assert_eq!(color, Color::new(1, 162, 255));
}

#[test]
fn from_hex_invalid_length_defaults_white() {
    let color = Color::from_hex("#fff");
    assert_eq!(color, Color::white());
}

#[test]
fn from_hex_invalid_component_defaults_to_255() {
    let color = Color::from_hex("zz00aa");
    assert_eq!(color, Color::new(255, 0, 170));
}

#[test]
fn ansi_sequences_match_rgb() {
    let color = Color::new(10, 20, 30);
    assert_eq!(color.fg_ansi(), "\x1b[38;2;10;20;30m");
    assert_eq!(color.bg_ansi(), "\x1b[48;2;10;20;30m");
}

#[test]
fn colorize_helpers_wrap_with_reset() {
    let text = "hi";
    let fg = Color::new(1, 2, 3);
    let bg = Color::new(4, 5, 6);

    let fg_only = colorize(text, fg);
    assert_eq!(fg_only, "\x1b[38;2;1;2;3mhi\x1b[0m");

    let fg_bg = colorize_bg(text, fg, bg);
    assert_eq!(fg_bg, "\x1b[38;2;1;2;3m\x1b[48;2;4;5;6mhi\x1b[0m");
}
