use hyprlog::Config;
use hyprlog::Error;
use std::fs;
use tempfile::TempDir;

#[test]
fn load_with_sources_merges_maps() {
    let tmp_dir = TempDir::new().unwrap();
    let base_path = tmp_dir.path().join("base.toml");
    let child_path = tmp_dir.path().join("child.toml");

    let base_content = format!(
        r##"
source = "{}"

[colors]
red = "#ff0000"
"##,
        child_path.display()
    );
    fs::write(&base_path, base_content).unwrap();

    fs::write(
        &child_path,
        r##"
[colors]
blue = "#0000ff"

[presets.test]
level = "info"
scope = "S"
msg = "M"
"##,
    )
    .unwrap();

    let config = Config::load_from(&base_path).unwrap();
    assert!(config.colors.contains_key("red"));
    assert!(config.colors.contains_key("blue"));
    assert!(config.presets.contains_key("test"));
}

#[test]
fn load_with_missing_source_is_ignored() {
    let tmp_dir = TempDir::new().unwrap();
    let base_path = tmp_dir.path().join("base.toml");

    let base_content = format!(
        r##"
source = "{}"

[colors]
red = "#ff0000"
"##,
        tmp_dir.path().join("missing.toml").display()
    );
    fs::write(&base_path, base_content).unwrap();

    let config = Config::load_from(&base_path).unwrap();
    assert!(config.colors.contains_key("red"));
}

#[test]
fn load_with_cyclic_sources_errors() {
    let tmp_dir = TempDir::new().unwrap();
    let a_path = tmp_dir.path().join("a.toml");
    let b_path = tmp_dir.path().join("b.toml");

    let a_content = format!(r#"source = "{}""#, b_path.display());
    let b_content = format!(r#"source = "{}""#, a_path.display());
    fs::write(&a_path, a_content).unwrap();
    fs::write(&b_path, b_content).unwrap();

    let err = Config::load_from(&a_path).unwrap_err();
    assert!(matches!(err, Error::CyclicInclude(_)));
}
