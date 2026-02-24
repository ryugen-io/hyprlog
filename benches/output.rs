use criterion::{Criterion, criterion_group, criterion_main};
use hyprlog::fmt::FormatValues;
use hyprlog::level::Level;
use hyprlog::output::{FileOutput, JsonOutput, LogRecord, Output};
use std::hint::black_box;
use tempfile::TempDir;

fn make_record() -> LogRecord {
    LogRecord {
        level: Level::Info,
        scope: "BENCH".to_string(),
        message: "benchmark log message with <bold>styling</bold>".to_string(),
        values: FormatValues::new()
            .timestamp("2025-01-15 14:30:00")
            .tag("[INFO ]")
            .scope("BENCH       ")
            .msg("benchmark log message with styling"),
        label_override: None,
        app_name: Some("bench-app".to_string()),
        raw: false,
    }
}

fn bench_file_output_write(c: &mut Criterion) {
    let tmp = TempDir::new().expect("failed to create temp dir");
    let output = FileOutput::new().base_dir(tmp.path().to_string_lossy().to_string());
    let record = make_record();

    c.bench_function("FileOutput::write", |b| {
        b.iter(|| {
            output.write(black_box(&record)).expect("write failed");
        });
    });

    output.flush().expect("flush failed");
}

fn bench_json_output_write(c: &mut Criterion) {
    let tmp = TempDir::new().expect("failed to create temp dir");
    let path = tmp.path().join("bench.jsonl");
    let output = JsonOutput::new().path(&path);
    let record = make_record();

    c.bench_function("JsonOutput::write", |b| {
        b.iter(|| {
            output.write(black_box(&record)).expect("write failed");
        });
    });
}

criterion_group!(benches, bench_file_output_write, bench_json_output_write,);
criterion_main!(benches);
