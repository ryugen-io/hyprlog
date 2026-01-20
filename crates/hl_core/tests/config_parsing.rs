use hl_core::Config;
use hl_core::fmt::{Alignment, IconType, Transform};

#[test]
fn parse_icon_type_variants() {
    let mut config = Config::default();

    config.terminal.icons = "ascii".to_string();
    assert_eq!(config.parse_icon_type(), IconType::Ascii);

    config.terminal.icons = "none".to_string();
    assert_eq!(config.parse_icon_type(), IconType::None);

    config.terminal.icons = "nerdfont".to_string();
    assert_eq!(config.parse_icon_type(), IconType::NerdFont);
}

#[test]
fn parse_alignment_variants() {
    let mut config = Config::default();

    config.tag.alignment = "left".to_string();
    assert_eq!(config.parse_alignment(), Alignment::Left);

    config.tag.alignment = "right".to_string();
    assert_eq!(config.parse_alignment(), Alignment::Right);

    config.tag.alignment = "center".to_string();
    assert_eq!(config.parse_alignment(), Alignment::Center);
}

#[test]
fn parse_scope_and_message_transforms() {
    let mut config = Config::default();

    config.scope.transform = "cap".to_string();
    assert_eq!(config.parse_scope_transform(), Transform::Capitalize);

    config.scope.transform = "lower".to_string();
    assert_eq!(config.parse_scope_transform(), Transform::Lowercase);

    config.message.transform = "upper".to_string();
    assert_eq!(config.parse_message_transform(), Transform::Uppercase);
}

#[test]
fn parse_scope_alignment_variants() {
    let mut config = Config::default();

    config.scope.alignment = "left".to_string();
    assert_eq!(config.parse_scope_alignment(), Alignment::Left);

    config.scope.alignment = "right".to_string();
    assert_eq!(config.parse_scope_alignment(), Alignment::Right);

    config.scope.alignment = "center".to_string();
    assert_eq!(config.parse_scope_alignment(), Alignment::Center);
}
