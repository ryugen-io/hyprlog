use criterion::{Criterion, criterion_group, criterion_main};
use hyprlog::config::HighlightConfig;
use hyprlog::fmt::style;
use hyprlog::fmt::{
    Alignment, Color, FormatTemplate, FormatValues, ScopeConfig, TagConfig, Transform, inject_tags,
};
use hyprlog::level::Level;
use std::collections::HashMap;
use std::hint::black_box;

fn bench_format_template_render(c: &mut Criterion) {
    let template = FormatTemplate::parse("{timestamp} {tag} {scope}  {msg}");
    let values = FormatValues::new()
        .timestamp("2025-01-15 14:30:00")
        .tag("[INFO ]")
        .scope("MAIN        ")
        .msg("Application started successfully");

    c.bench_function("FormatTemplate::render", |b| {
        b.iter(|| template.render(black_box(&values)));
    });
}

fn bench_tag_format(c: &mut Criterion) {
    let mut group = c.benchmark_group("TagConfig::format");

    let config = TagConfig::default();
    group.bench_function("default", |b| {
        b.iter(|| config.format(black_box(Level::Info)));
    });

    let config_custom = TagConfig::new()
        .prefix("<<")
        .suffix(">>")
        .transform(Transform::Capitalize)
        .min_width(10)
        .alignment(Alignment::Right);
    group.bench_function("custom", |b| {
        b.iter(|| config_custom.format(black_box(Level::Warn)));
    });

    group.finish();
}

fn bench_style_render(c: &mut Criterion) {
    let mut colors = HashMap::new();
    colors.insert("red".to_string(), Color::red());
    colors.insert("cyan".to_string(), Color::cyan());
    colors.insert("green".to_string(), Color::green());

    let segments =
        style::parse("<red>Error:</red> file <bold>not found</bold> at <cyan>/tmp/test</cyan>");

    c.bench_function("style::render", |b| {
        b.iter(|| style::render(black_box(&segments), black_box(&colors)));
    });
}

fn bench_highlight_inject_tags(c: &mut Criterion) {
    let mut config = HighlightConfig {
        enabled: true,
        ..HighlightConfig::default()
    };
    config.patterns.urls = Some("cyan".to_string());
    config.patterns.paths = Some("green".to_string());
    config.patterns.numbers = Some("yellow".to_string());
    config.patterns.quoted = Some("orange".to_string());
    config
        .keywords
        .insert("error".to_string(), "red".to_string());
    config
        .keywords
        .insert("warning".to_string(), "yellow".to_string());
    config
        .keywords
        .insert("success".to_string(), "green".to_string());

    let mut group = c.benchmark_group("highlight::inject_tags");

    group.bench_function("short", |b| {
        b.iter(|| inject_tags(black_box("simple message"), black_box(&config)));
    });

    group.bench_function("with_url_and_path", |b| {
        b.iter(|| {
            inject_tags(
                black_box("fetching https://api.example.com/v2/data from /var/log/app.log"),
                black_box(&config),
            )
        });
    });

    group.bench_function("keywords_heavy", |b| {
        b.iter(|| {
            inject_tags(
                black_box("error in module: warning count 42, success rate 0.95 at /tmp/out"),
                black_box(&config),
            )
        });
    });

    group.bench_function("long_mixed", |b| {
        b.iter(|| {
            inject_tags(
                black_box(
                    "Request error: GET https://api.example.com/users/123 returned 500 \
                     at /var/log/api/2025-01-15.log - \"Internal Server Error\" with \
                     warning: retry count 3, success rate dropped to 0.42",
                ),
                black_box(&config),
            )
        });
    });

    group.finish();
}

fn bench_scope_format(c: &mut Criterion) {
    let mut group = c.benchmark_group("ScopeConfig::format");

    let config = ScopeConfig::default();
    group.bench_function("default", |b| {
        b.iter(|| config.format(black_box("MAIN")));
    });

    let config_custom = ScopeConfig::new()
        .min_width(20)
        .alignment(Alignment::Center)
        .transform(Transform::Uppercase);
    group.bench_function("custom", |b| {
        b.iter(|| config_custom.format(black_box("network")));
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_format_template_render,
    bench_tag_format,
    bench_style_render,
    bench_highlight_inject_tags,
    bench_scope_format,
);
criterion_main!(benches);
