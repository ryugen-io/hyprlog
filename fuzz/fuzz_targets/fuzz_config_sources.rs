#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    // Must not panic on any config content
    let _ = hyprlog::config::extract_sources(data);
});
