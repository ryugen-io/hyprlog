use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_hyprslog"))
        .args(args)
        .output()
        .expect("failed to run hyprslog")
}

#[test]
fn help_shows_usage() {
    let output = run(&["help"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("hyprslog"));
}

#[test]
fn version_prints_version_string() {
    let output = run(&["version"]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with("hyprslog "));
}

#[test]
fn unknown_command_exits_failure() {
    let output = run(&["does-not-exist"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unknown command"));
}
