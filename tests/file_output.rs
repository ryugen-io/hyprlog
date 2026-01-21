use hyprlog::Logger;
use std::fs;
use tempfile::TempDir;

#[test]
fn file_output_writes_stripped_message() {
    let tmp_dir = TempDir::new().unwrap();
    let base_dir = tmp_dir.path().to_string_lossy().into_owned();

    let logger = Logger::builder()
        .file()
        .base_dir(base_dir)
        .path_structure("logs")
        .filename_structure("test.log")
        .content_structure("{scope}|{level}|{msg}")
        .done()
        .build();

    logger.info("SC", "Hello <bold>World</bold>");

    let path = tmp_dir.path().join("logs").join("test.log");
    let content = fs::read_to_string(path).unwrap();

    assert_eq!(content.trim(), "SC|info|Hello World");
}

#[test]
fn file_output_uses_app_name_in_path() {
    let tmp_dir = TempDir::new().unwrap();
    let base_dir = tmp_dir.path().to_string_lossy().into_owned();

    let logger = Logger::builder()
        .file()
        .base_dir(base_dir)
        .path_structure("{app}")
        .filename_structure("out.log")
        .content_structure("{scope}:{msg}")
        .app_name("myapp")
        .done()
        .build();

    logger.info("S", "Test");

    let path = tmp_dir.path().join("myapp").join("out.log");
    assert!(path.exists());
}

#[test]
fn file_output_appends_multiple_lines() {
    let tmp_dir = TempDir::new().unwrap();
    let base_dir = tmp_dir.path().to_string_lossy().into_owned();

    let logger = Logger::builder()
        .file()
        .base_dir(base_dir)
        .path_structure("logs")
        .filename_structure("multi.log")
        .content_structure("{msg}")
        .done()
        .build();

    logger.info("S", "one");
    logger.info("S", "two");

    let path = tmp_dir.path().join("logs").join("multi.log");
    let content = fs::read_to_string(path).unwrap();
    let lines: Vec<&str> = content.lines().collect();

    assert_eq!(lines, vec!["one", "two"]);
}

#[test]
fn file_output_uses_log_full_app_override() {
    let tmp_dir = TempDir::new().unwrap();
    let base_dir = tmp_dir.path().to_string_lossy().into_owned();

    let logger = Logger::builder()
        .file()
        .base_dir(base_dir)
        .path_structure("{app}")
        .filename_structure("override.log")
        .content_structure("{msg}")
        .done()
        .build();

    logger.log_full(hyprlog::Level::Info, "S", "Override", Some("appx"));

    let path = tmp_dir.path().join("appx").join("override.log");
    assert!(path.exists());
}
