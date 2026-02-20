#![no_main]
use hyprlog::hyprland::event::HyprlandEvent;
use hyprlog::hyprland::formatter::EventFormatter;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    // Split fuzz input into name and data at first comma
    let (name, event_data) = data.split_once(',').unwrap_or((data, ""));

    let event = HyprlandEvent {
        name: name.to_string(),
        data: event_data.to_string(),
    };

    // Must not panic in human-readable mode
    let mut formatter = EventFormatter::new(true);
    let _ = formatter.format(&event);
    formatter.observe(&event);

    // Format again after observe (cache may be populated)
    let _ = formatter.format(&event);

    // Must not panic in raw mode either
    let raw = EventFormatter::new(false);
    let _ = raw.format(&event);
});
