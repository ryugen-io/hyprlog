#![no_main]
use libfuzzer_sys::fuzz_target;
use hyprlog::fmt::{FormatTemplate, FormatValues};

fuzz_target!(|data: &str| {
    // Must not panic on any template string
    let template = FormatTemplate::parse(data);

    // Also exercise rendering with values
    let values = FormatValues::new()
        .tag("INFO")
        .scope("MAIN")
        .msg("test")
        .timestamp("2025-01-01 00:00:00")
        .level("info")
        .app("fuzz")
        .date("2025", "01", "01");
    let _ = template.render(&values);
});
