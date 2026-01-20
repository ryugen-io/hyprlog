use hl_core::fmt::{FormatSegment, FormatTemplate, FormatValues, Placeholder};

#[test]
fn parse_mixed_placeholders_and_literals() {
    let template = FormatTemplate::parse("A{tag}B{unknown}C{msg}D");
    let segments = template.segments();

    assert_eq!(
        segments,
        &[
            FormatSegment::Literal("A".to_string()),
            FormatSegment::Placeholder(Placeholder::Tag),
            FormatSegment::Literal("B".to_string()),
            FormatSegment::Literal("{unknown}".to_string()),
            FormatSegment::Literal("C".to_string()),
            FormatSegment::Placeholder(Placeholder::Msg),
            FormatSegment::Literal("D".to_string()),
        ]
    );
}

#[test]
fn render_replaces_known_placeholders() {
    let template = FormatTemplate::parse("{tag}-{msg}");
    let values = FormatValues::new().tag("T").msg("M");

    assert_eq!(template.render(&values), "T-M");
}

#[test]
fn parse_unclosed_brace_is_literal() {
    let template = FormatTemplate::parse("start {tag");
    let values = FormatValues::new().tag("T");

    assert_eq!(template.render(&values), "start {tag");
}
