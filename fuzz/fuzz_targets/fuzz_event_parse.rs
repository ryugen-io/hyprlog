#![no_main]
use libfuzzer_sys::fuzz_target;
use hyprlog::hyprland::event::HyprlandEvent;

fuzz_target!(|data: &str| {
    // Must not panic on any input
    let _ = HyprlandEvent::parse(data);
});
