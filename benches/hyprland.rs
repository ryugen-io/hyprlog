use criterion::{Criterion, black_box, criterion_group, criterion_main};
use hyprlog::hyprland::event::HyprlandEvent;
use hyprlog::hyprland::formatter::EventFormatter;
use hyprlog::hyprland::level_map;
use std::collections::HashMap;

fn make_event(name: &str, data: &str) -> HyprlandEvent {
    HyprlandEvent {
        name: name.to_string(),
        data: data.to_string(),
    }
}

fn bench_format_human_readable(c: &mut Criterion) {
    let mut group = c.benchmark_group("EventFormatter::format_human");

    let formatter = EventFormatter::new(true);

    group.bench_function("openwindow", |b| {
        let event = make_event("openwindow", "55e3f120,2,kitty,~");
        b.iter(|| formatter.format(black_box(&event)));
    });

    group.bench_function("workspace", |b| {
        let event = make_event("workspace", "3");
        b.iter(|| formatter.format(black_box(&event)));
    });

    group.bench_function("activewindow", |b| {
        let event = make_event("activewindow", "kitty,~/projects");
        b.iter(|| formatter.format(black_box(&event)));
    });

    group.bench_function("empty_data", |b| {
        let event = make_event("configreloaded", "");
        b.iter(|| formatter.format(black_box(&event)));
    });

    group.bench_function("unknown_event", |b| {
        let event = make_event("customevent", "some,arbitrary,data");
        b.iter(|| formatter.format(black_box(&event)));
    });

    group.finish();
}

fn bench_format_raw(c: &mut Criterion) {
    let formatter = EventFormatter::new(false);
    let event = make_event("openwindow", "55e3f120,2,kitty,~");

    c.bench_function("EventFormatter::format_raw", |b| {
        b.iter(|| formatter.format(black_box(&event)));
    });
}

fn bench_format_with_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("EventFormatter::format_cached");

    // Warm cache: observe openwindow first, then format closewindow
    let mut formatter = EventFormatter::new(true);
    let open = make_event("openwindow", "55e3f120,2,kitty,~");
    formatter.observe(&open);

    group.bench_function("closewindow_cached", |b| {
        let event = make_event("closewindow", "55e3f120");
        b.iter(|| formatter.format(black_box(&event)));
    });

    group.bench_function("urgent_cached", |b| {
        let event = make_event("urgent", "55e3f120");
        b.iter(|| formatter.format(black_box(&event)));
    });

    group.bench_function("closewindow_uncached", |b| {
        let cold = EventFormatter::new(true);
        let event = make_event("closewindow", "deadbeef");
        b.iter(|| cold.format(black_box(&event)));
    });

    group.finish();
}

fn bench_observe(c: &mut Criterion) {
    let mut group = c.benchmark_group("EventFormatter::observe");

    group.bench_function("openwindow", |b| {
        let mut formatter = EventFormatter::new(true);
        let event = make_event("openwindow", "55e3f120,2,kitty,~");
        b.iter(|| {
            formatter.observe(black_box(&event));
        });
    });

    group.bench_function("closewindow", |b| {
        let mut formatter = EventFormatter::new(true);
        let event = make_event("closewindow", "55e3f120");
        b.iter(|| {
            formatter.observe(black_box(&event));
        });
    });

    group.bench_function("irrelevant_event", |b| {
        let mut formatter = EventFormatter::new(true);
        let event = make_event("workspace", "3");
        b.iter(|| {
            formatter.observe(black_box(&event));
        });
    });

    group.finish();
}

fn bench_resolve_level(c: &mut Criterion) {
    let mut group = c.benchmark_group("level_map::resolve_level");
    let empty: HashMap<String, String> = HashMap::new();
    let mut overrides = HashMap::new();
    overrides.insert("openwindow".to_string(), "debug".to_string());

    group.bench_function("default_hit", |b| {
        b.iter(|| level_map::resolve_level(black_box("openwindow"), black_box(&empty)));
    });

    group.bench_function("default_miss", |b| {
        b.iter(|| level_map::resolve_level(black_box("customevent"), black_box(&empty)));
    });

    group.bench_function("user_override", |b| {
        b.iter(|| level_map::resolve_level(black_box("openwindow"), black_box(&overrides)));
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_format_human_readable,
    bench_format_raw,
    bench_format_with_cache,
    bench_observe,
    bench_resolve_level,
);
criterion_main!(benches);
