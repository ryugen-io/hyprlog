#![cfg(feature = "cli")]

use hyprlog::Config;
use hyprlog::Logger;
use hyprlog::cli::PresetRunner;
use hyprlog::config::PresetConfig;

#[test]
fn run_returns_error_for_invalid_level() {
    let mut config = Config::default();
    config.presets.insert(
        "bad".to_string(),
        PresetConfig {
            level: "nope".to_string(),
            as_level: None,
            scope: Some("S".to_string()),
            msg: "M".to_string(),
            app_name: None,
        },
    );
    let logger = Logger::builder().build();
    let runner = PresetRunner::new(&config, &logger);

    let result = runner.run("bad");
    assert!(result.is_err());
}

#[test]
fn run_ok_and_list_contains_entry() {
    let mut config = Config::default();
    config.presets.insert(
        "ok".to_string(),
        PresetConfig {
            level: "info".to_string(),
            as_level: None,
            scope: None,
            msg: "hello".to_string(),
            app_name: Some("app".to_string()),
        },
    );
    let logger = Logger::builder().build();
    let runner = PresetRunner::new(&config, &logger);

    assert!(runner.exists("ok"));
    assert!(runner.run("ok").is_ok());

    let list = runner.list();
    assert!(
        list.iter()
            .any(|(name, app)| *name == "ok" && *app == Some("app"))
    );
}
