use criterion::{Criterion, black_box, criterion_group, criterion_main};
use hyprlog::fmt::Color;
use hyprlog::fmt::FormatTemplate;
use hyprlog::fmt::style;
use hyprlog::level::Level;
use std::str::FromStr;

fn bench_format_template_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("FormatTemplate::parse");

    group.bench_function("simple", |b| {
        b.iter(|| FormatTemplate::parse(black_box("{tag} {scope}  {msg}")));
    });

    group.bench_function("all_placeholders", |b| {
        b.iter(|| {
            FormatTemplate::parse(black_box(
                "{timestamp} {icon} {tag} {level} {scope} {app} {year}/{month}/{day} {msg}",
            ))
        });
    });

    group.bench_function("literal_only", |b| {
        b.iter(|| FormatTemplate::parse(black_box("no placeholders here at all")));
    });

    group.bench_function("unknown_placeholders", |b| {
        b.iter(|| FormatTemplate::parse(black_box("{foo} {bar} {baz} {tag}")));
    });

    group.finish();
}

fn bench_color_from_hex(c: &mut Criterion) {
    let mut group = c.benchmark_group("Color::from_hex");

    group.bench_function("valid_with_hash", |b| {
        b.iter(|| Color::from_hex(black_box("#ff5555")));
    });

    group.bench_function("valid_without_hash", |b| {
        b.iter(|| Color::from_hex(black_box("50fa7b")));
    });

    group.bench_function("invalid_short", |b| {
        b.iter(|| Color::from_hex(black_box("#fff")));
    });

    group.bench_function("invalid_chars", |b| {
        b.iter(|| Color::from_hex(black_box("#zzzzzz")));
    });

    group.finish();
}

fn bench_level_from_str(c: &mut Criterion) {
    let mut group = c.benchmark_group("Level::from_str");

    group.bench_function("valid_info", |b| {
        b.iter(|| Level::from_str(black_box("info")));
    });

    group.bench_function("valid_warning", |b| {
        b.iter(|| Level::from_str(black_box("warning")));
    });

    group.bench_function("invalid", |b| {
        b.iter(|| Level::from_str(black_box("critical")));
    });

    group.finish();
}

fn bench_style_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("style::parse");

    group.bench_function("plain", |b| {
        b.iter(|| style::parse(black_box("no tags here")));
    });

    group.bench_function("single_tag", |b| {
        b.iter(|| style::parse(black_box("hello <bold>world</bold>")));
    });

    group.bench_function("nested_multiple", |b| {
        b.iter(|| {
            style::parse(black_box(
                "<red>Error:</red> file <bold>not found</bold> at <cyan>/tmp/test</cyan>",
            ))
        });
    });

    group.bench_function("many_tags", |b| {
        b.iter(|| {
            style::parse(black_box(
                "<bold>a</bold> <dim>b</dim> <italic>c</italic> <underline>d</underline> \
                 <red>e</red> <green>f</green> <blue>g</blue> <yellow>h</yellow>",
            ))
        });
    });

    group.finish();
}

fn bench_extract_sources(c: &mut Criterion) {
    let config_with_sources = "\
source = \"~/.config/hypr/colors.conf\"
source = \"~/.config/hypr/presets.conf\"

[general]
level = \"info\"

[terminal]
enabled = true
colors = true

source = \"/etc/hyprlog/defaults.conf\"

[tag]
prefix = \"[\"
suffix = \"]\"
";

    let config_no_sources = "\
[general]
level = \"info\"

[terminal]
enabled = true
colors = true

[tag]
prefix = \"[\"
suffix = \"]\"
";

    let mut group = c.benchmark_group("extract_sources");

    group.bench_function("with_sources", |b| {
        b.iter(|| hyprlog::config::extract_sources(black_box(config_with_sources)));
    });

    group.bench_function("no_sources", |b| {
        b.iter(|| hyprlog::config::extract_sources(black_box(config_no_sources)));
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_format_template_parse,
    bench_color_from_hex,
    bench_level_from_str,
    bench_style_parse,
    bench_extract_sources,
);
criterion_main!(benches);
