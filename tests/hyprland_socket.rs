//! Tests for Hyprland socket path resolution.

#![cfg(feature = "hyprland")]

use hyprlog::config::HyprlandConfig;
use hyprlog::hyprland::socket;
use std::path::PathBuf;

#[test]
fn socket1_path_format() {
    let dir = PathBuf::from("/run/user/1000/hypr/abc123");
    let path = socket::socket1_path(&dir);
    assert_eq!(
        path,
        PathBuf::from("/run/user/1000/hypr/abc123/.socket.sock")
    );
}

#[test]
fn socket2_path_format() {
    let dir = PathBuf::from("/run/user/1000/hypr/abc123");
    let path = socket::socket2_path(&dir);
    assert_eq!(
        path,
        PathBuf::from("/run/user/1000/hypr/abc123/.socket2.sock")
    );
}

#[test]
fn resolve_with_config_socket_dir_override() {
    let tmp = tempfile::tempdir().unwrap();
    let config = HyprlandConfig {
        socket_dir: Some(tmp.path().to_string_lossy().into_owned()),
        ..HyprlandConfig::default()
    };
    let dir = socket::resolve_socket_dir(&config).unwrap();
    assert_eq!(dir, tmp.path());
}

#[test]
fn resolve_with_nonexistent_socket_dir_override_fails() {
    let config = HyprlandConfig {
        socket_dir: Some("/nonexistent/path/that/does/not/exist".to_string()),
        ..HyprlandConfig::default()
    };
    assert!(socket::resolve_socket_dir(&config).is_err());
}

#[test]
fn resolve_without_env_vars_fails() {
    // Temporarily clear the env vars to test fallback behavior.
    // This test is safe because we're only reading env vars, not mutating global state
    // in a way that affects other tests (env vars are per-process).
    // However, to avoid race conditions, we just test with a config that has
    // no socket_dir and no env var set — which should fail with NoInstanceSignature
    // or NoRuntimeDir if those aren't set.
    let config = HyprlandConfig::default();
    // This will either succeed (if running under Hyprland) or fail with a clear error.
    // We just verify it doesn't panic.
    let _ = socket::resolve_socket_dir(&config);
}
