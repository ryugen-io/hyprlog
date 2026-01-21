use hyprlog::fmt::{Color, Segment, parse, render, render_plain, strip_tags};
use std::collections::HashMap;

#[test]
fn parse_splits_plain_and_styled_segments() {
    let segments = parse("hello <bold>world</bold>!");
    assert_eq!(
        segments,
        vec![
            Segment::Plain("hello ".to_string()),
            Segment::Bold("world".to_string()),
            Segment::Plain("!".to_string()),
        ]
    );
}

#[test]
fn parse_color_tag_as_colored_segment() {
    let segments = parse("<red>hi</red>");
    assert_eq!(
        segments,
        vec![Segment::Colored("hi".to_string(), "red".to_string())]
    );
}

#[test]
fn render_uses_named_color_map() {
    let segments = parse("<red>hi</red>");
    let mut colors = HashMap::new();
    colors.insert("red".to_string(), Color::new(1, 2, 3));

    assert_eq!(render(&segments, &colors), "\x1b[38;2;1;2;3mhi\x1b[0m");
}

#[test]
fn render_plain_and_strip_tags_remove_styles() {
    let segments = parse("a<bold>b</bold>c");
    assert_eq!(render_plain(&segments), "abc");
    assert_eq!(strip_tags("a<bold>b</bold>c"), "abc");
}
