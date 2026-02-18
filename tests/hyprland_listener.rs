//! Integration tests for the Hyprland listener loop.

#![cfg(feature = "hyprland")]

use hyprlog::Logger;
use hyprlog::config::HyprlandConfig;
use hyprlog::hyprland::listener;
use hyprlog::level::Level;
use hyprlog::output::{LogRecord, Output};
use std::io::Write;
use std::os::unix::net::UnixListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone)]
struct CaptureOutput {
    records: Arc<Mutex<Vec<LogRecord>>>,
}

impl Output for CaptureOutput {
    fn write(&self, record: &LogRecord) -> Result<(), hyprlog::Error> {
        self.records
            .lock()
            .expect("records lock poisoned")
            .push(record.clone());
        Ok(())
    }

    fn flush(&self) -> Result<(), hyprlog::Error> {
        Ok(())
    }
}

fn wait_for_records(
    records: &Arc<Mutex<Vec<LogRecord>>>,
    expected_min: usize,
    timeout: Duration,
) -> bool {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if records.lock().expect("records lock poisoned").len() >= expected_min {
            return true;
        }
        thread::sleep(Duration::from_millis(10));
    }
    false
}

#[test]
fn run_event_loop_logs_events_and_applies_allowlist_filter() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let socket_path = tmp.path().join(".socket2.sock");
    let listener_socket = UnixListener::bind(&socket_path).expect("bind socket2");

    let server_thread = thread::spawn(move || {
        let (mut stream, _) = listener_socket.accept().expect("accept socket2 client");
        writeln!(stream, "openwindow>>80a6f50,2,kitty,Kitty").expect("write openwindow");
        writeln!(stream, "closewindow>>80a6f50").expect("write closewindow");
        stream.flush().expect("flush stream");
    });

    let records = Arc::new(Mutex::new(Vec::<LogRecord>::new()));
    let logger = Logger::builder()
        .output(CaptureOutput {
            records: Arc::clone(&records),
        })
        .build();

    let mut config = HyprlandConfig {
        scope: "HYPRTEST".to_string(),
        ..HyprlandConfig::default()
    };
    config.event_filter = Some(vec!["openwindow".to_string()]);

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_for_loop = Arc::clone(&shutdown);
    let socket_dir = tmp.path().to_path_buf();

    let loop_thread = thread::spawn(move || {
        listener::run_event_loop(&socket_dir, &logger, &config, &shutdown_for_loop);
    });

    assert!(
        wait_for_records(&records, 1, Duration::from_secs(3)),
        "expected at least one logged event"
    );

    thread::sleep(Duration::from_millis(100));
    shutdown.store(true, Ordering::Relaxed);

    loop_thread.join().expect("listener thread should join");
    server_thread.join().expect("server thread should join");

    let captured = records.lock().expect("records lock poisoned");
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0].level, Level::Info);
    assert_eq!(captured[0].scope, "kitty");
    assert_eq!(captured[0].message, "openwindow: 80a6f50,2,kitty,Kitty");
}

#[test]
fn run_event_loop_custom_events_use_hyprctl_scope() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let socket_path = tmp.path().join(".socket2.sock");
    let listener_socket = UnixListener::bind(&socket_path).expect("bind socket2");

    let server_thread = thread::spawn(move || {
        let (mut stream, _) = listener_socket.accept().expect("accept socket2 client");
        writeln!(stream, "custom>>from_hyprctl").expect("write custom");
        stream.flush().expect("flush stream");
    });

    let records = Arc::new(Mutex::new(Vec::<LogRecord>::new()));
    let logger = Logger::builder()
        .output(CaptureOutput {
            records: Arc::clone(&records),
        })
        .build();

    let mut config = HyprlandConfig {
        scope: "HYPRTEST".to_string(),
        ..HyprlandConfig::default()
    };
    config.event_filter = Some(vec!["custom".to_string()]);

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_for_loop = Arc::clone(&shutdown);
    let socket_dir = tmp.path().to_path_buf();

    let loop_thread = thread::spawn(move || {
        listener::run_event_loop(&socket_dir, &logger, &config, &shutdown_for_loop);
    });

    assert!(
        wait_for_records(&records, 1, Duration::from_secs(3)),
        "expected at least one logged event"
    );

    thread::sleep(Duration::from_millis(100));
    shutdown.store(true, Ordering::Relaxed);

    loop_thread.join().expect("listener thread should join");
    server_thread.join().expect("server thread should join");

    let captured = records.lock().expect("records lock poisoned");
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0].scope, "hyprctl");
    assert_eq!(captured[0].message, "custom: from_hyprctl");
}

#[test]
fn run_event_loop_applies_event_scope_overrides() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let socket_path = tmp.path().join(".socket2.sock");
    let listener_socket = UnixListener::bind(&socket_path).expect("bind socket2");

    let server_thread = thread::spawn(move || {
        let (mut stream, _) = listener_socket.accept().expect("accept socket2 client");
        writeln!(stream, "openwindow>>80a6f50,2,kitty,Kitty").expect("write openwindow");
        stream.flush().expect("flush stream");
    });

    let records = Arc::new(Mutex::new(Vec::<LogRecord>::new()));
    let logger = Logger::builder()
        .output(CaptureOutput {
            records: Arc::clone(&records),
        })
        .build();

    let mut config = HyprlandConfig {
        scope: "HYPRTEST".to_string(),
        ..HyprlandConfig::default()
    };
    config
        .event_scopes
        .insert("openwindow".to_string(), "window.app".to_string());
    config.event_filter = Some(vec!["openwindow".to_string()]);

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_for_loop = Arc::clone(&shutdown);
    let socket_dir = tmp.path().to_path_buf();

    let loop_thread = thread::spawn(move || {
        listener::run_event_loop(&socket_dir, &logger, &config, &shutdown_for_loop);
    });

    assert!(
        wait_for_records(&records, 1, Duration::from_secs(3)),
        "expected at least one logged event"
    );

    thread::sleep(Duration::from_millis(100));
    shutdown.store(true, Ordering::Relaxed);

    loop_thread.join().expect("listener thread should join");
    server_thread.join().expect("server thread should join");

    let captured = records.lock().expect("records lock poisoned");
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0].scope, "window.app");
}
