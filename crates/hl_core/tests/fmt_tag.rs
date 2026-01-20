use hl_core::Level;
use hl_core::fmt::{Alignment, TagConfig, Transform};

#[test]
fn format_applies_alignment_and_width() {
    let tag = TagConfig::default()
        .min_width(6)
        .alignment(Alignment::Left)
        .transform(Transform::None)
        .prefix("<")
        .suffix(">")
        .format(Level::Info);

    assert_eq!(tag, "<info  >");
}

#[test]
fn format_right_alignment_padding() {
    let tag = TagConfig::default()
        .min_width(6)
        .alignment(Alignment::Right)
        .transform(Transform::None)
        .prefix("")
        .suffix("")
        .format(Level::Info);

    assert_eq!(tag, "  info");
}

#[test]
fn format_center_alignment_padding() {
    let tag = TagConfig::default()
        .min_width(7)
        .alignment(Alignment::Center)
        .transform(Transform::None)
        .prefix("")
        .suffix("")
        .format(Level::Info);

    assert_eq!(tag, " info  ");
}

#[test]
fn format_applies_transform() {
    let tag = TagConfig::default()
        .transform(Transform::Lowercase)
        .min_width(0)
        .prefix("")
        .suffix("")
        .format(Level::Warn);

    assert_eq!(tag, "warn");
}
