//! Tests for auto-highlighting functionality.

use hl_core::config::HighlightConfig;
use hl_core::fmt::highlight::inject_tags;

fn test_config() -> HighlightConfig {
    HighlightConfig::default()
}

#[test]
fn test_disabled() {
    let mut config = test_config();
    config.enabled = false;
    let msg = "Status: OK";
    assert_eq!(inject_tags(msg, &config), msg);
}

#[test]
fn test_empty_message() {
    let config = test_config();
    assert_eq!(inject_tags("", &config), "");
}

#[test]
fn test_keyword_highlighting() {
    let config = test_config();
    let result = inject_tags("Status: OK", &config);
    assert!(result.contains("<green>OK</green>"));
}

#[test]
fn test_keyword_case_insensitive() {
    let config = test_config();
    let result = inject_tags("error occurred", &config);
    assert!(result.contains("<red>error</red>"));
}

#[test]
fn test_url_highlighting() {
    let config = test_config();
    let result = inject_tags("Visit https://example.com for help", &config);
    assert!(result.contains("<blue>https://example.com</blue>"));
}

#[test]
fn test_path_highlighting() {
    let config = test_config();
    let result = inject_tags("File at /tmp/test.txt", &config);
    assert!(result.contains("<cyan>/tmp/test.txt</cyan>"));
}

#[test]
fn test_number_highlighting() {
    let config = test_config();
    let result = inject_tags("Count: 42", &config);
    assert!(result.contains("<orange>42</orange>"));
}

#[test]
fn test_quoted_highlighting() {
    let config = test_config();
    let result = inject_tags(r#"Value is "hello""#, &config);
    assert!(result.contains(r#"<yellow>"hello"</yellow>"#));
}

#[test]
fn test_skip_existing_tags() {
    let config = test_config();
    let msg = "Already <red>colored</red> OK";
    let result = inject_tags(msg, &config);
    // Should preserve existing tag and still highlight OK
    assert!(result.contains("<red>colored</red>"));
    assert!(result.contains("<green>OK</green>"));
}

#[test]
fn test_multiple_keywords() {
    let config = test_config();
    let result = inject_tags("OK and FAIL and true", &config);
    assert!(result.contains("<green>OK</green>"));
    assert!(result.contains("<red>FAIL</red>"));
    assert!(result.contains("<green>true</green>"));
}

#[test]
fn test_no_partial_word_match() {
    let config = test_config();
    // "LOOK" contains "OK" but should not match as a word boundary
    let result = inject_tags("LOOK at this", &config);
    assert!(!result.contains("<green>"));
}

#[test]
fn test_complex_message() {
    let config = test_config();
    let result = inject_tags("Status: OK, errors: 0, path: /tmp/log.txt", &config);
    assert!(result.contains("<green>OK</green>"));
    assert!(result.contains("<orange>0</orange>"));
    assert!(result.contains("<cyan>/tmp/log.txt</cyan>"));
}
