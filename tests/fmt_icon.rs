use hyprlog::Level;
use hyprlog::fmt::{IconSet, IconType};

#[test]
fn ascii_icons_expose_expected_values() {
    let icons = IconSet::ascii();

    assert_eq!(icons.icon_type(), IconType::Ascii);
    assert_eq!(icons.get(Level::Warn), "[!]");
    assert_eq!(icons.get(Level::Info), "[i]");
}

#[test]
fn none_icons_are_empty_until_set() {
    let mut icons = IconSet::none();

    assert_eq!(icons.icon_type(), IconType::None);
    assert_eq!(icons.get(Level::Info), "");

    icons.set(Level::Info, "i");
    assert_eq!(icons.get(Level::Info), "i");
}

#[test]
fn iconset_from_type_matches_variant() {
    let icons = IconSet::from(IconType::NerdFont);
    assert_eq!(icons.icon_type(), IconType::NerdFont);
    assert!(!icons.get(Level::Error).is_empty());
}
