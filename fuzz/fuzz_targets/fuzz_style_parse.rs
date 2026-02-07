#![no_main]
use libfuzzer_sys::fuzz_target;
use hyprlog::fmt::style;

fuzz_target!(|data: &str| {
    // Must not panic on any input, including unclosed/malformed tags
    let segments = style::parse(data);
    // Also exercise rendering to plain text
    let _ = style::render_plain(&segments);
});
