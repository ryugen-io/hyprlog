#![no_main]
use libfuzzer_sys::fuzz_target;
use hyprlog::config::HighlightConfig;
use hyprlog::fmt::inject_tags;

fuzz_target!(|data: &str| {
    let mut config = HighlightConfig::default();
    config.enabled = true;
    config.patterns.urls = Some("cyan".to_string());
    config.patterns.paths = Some("green".to_string());
    config.patterns.numbers = Some("yellow".to_string());
    config.patterns.quoted = Some("orange".to_string());
    config.keywords.insert("error".to_string(), "red".to_string());
    config.keywords.insert("warning".to_string(), "yellow".to_string());

    // Must not panic; exercises 5 regexes + overlap logic
    let _ = inject_tags(data, &config);
});
