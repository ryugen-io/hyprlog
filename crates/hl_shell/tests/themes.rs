use hl_shell::themes::{ALL_THEMES, Theme};
use std::str::FromStr;

#[test]
fn list_matches_all_themes() {
    let list = Theme::list();
    assert_eq!(list.len(), ALL_THEMES.len());
    for theme in ALL_THEMES {
        assert!(list.contains(&theme.name()));
    }
}

#[test]
fn from_str_accepts_aliases() {
    assert_eq!(Theme::from_str("tokyo-night").unwrap(), Theme::TokyoNight);
    assert_eq!(Theme::from_str("tokyo_night").unwrap(), Theme::TokyoNight);
    assert_eq!(Theme::from_str("tokyonight").unwrap(), Theme::TokyoNight);
}

#[test]
fn from_str_rejects_unknown() {
    assert!(Theme::from_str("unknown-theme").is_err());
}

#[test]
fn build_prompt_contains_text_and_reset() {
    let prompt = Theme::Dracula.build_prompt();
    assert!(prompt.contains('h'));
    assert!(prompt.contains('>'));
    assert!(prompt.ends_with("\x1b[0m "));
}
