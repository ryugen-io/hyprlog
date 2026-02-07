#![no_main]
use libfuzzer_sys::fuzz_target;
use hyprlog::fmt::Color;

fuzz_target!(|data: &str| {
    // Must not panic on any hex string
    let _ = Color::from_hex(data);
});
