//! The compositor fires events continuously — this module connects to socket2,
//! survives disconnects with exponential backoff, and feeds parsed events into the
//! logger without blocking the main thread.

use super::event::HyprlandEvent;
use super::formatter::EventFormatter;
use super::level_map::resolve_level;
use super::socket;
use crate::config::HyprlandConfig;
use crate::internal;
use crate::logger::Logger;
use hypr_sdk::ipc::EventStream;
use hypr_sdk::ipc::socket as ipc_socket;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tokio::runtime::Builder;
use tokio::time;

/// Without a handle, there's no way to cleanly stop the background thread —
/// dropping the handle signals shutdown so the listener doesn't outlive the shell.
pub struct EventListenerHandle {
    shutdown: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl EventListenerHandle {
    /// Cooperative shutdown — the loop checks this flag between events instead of being killed.
    pub fn stop(&self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }

    /// Blocking join ensures the thread has fully exited before the caller proceeds — prevents use-after-free on shared state.
    pub fn join(mut self) {
        self.stop();
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for EventListenerHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Blocking entry point — the background thread calls this directly. Reconnection with
/// exponential backoff handles compositor restarts and socket interruptions without
/// losing the listener permanently.
pub fn run_event_loop(
    socket_dir: &std::path::Path,
    logger: &Logger,
    config: &HyprlandConfig,
    shutdown: &AtomicBool,
) {
    let socket_path = socket::socket2_path(socket_dir);

    let runtime = match Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
    {
        Ok(runtime) => runtime,
        Err(e) => {
            internal::error(
                config.scope.as_str(),
                &format!("Failed to create Tokio runtime: {e}"),
            );
            return;
        }
    };

    runtime.block_on(async {
        run_event_loop_async(&socket_path, logger, config, shutdown).await;
    });
}

async fn run_event_loop_async(
    socket_path: &std::path::Path,
    logger: &Logger,
    config: &HyprlandConfig,
    shutdown: &AtomicBool,
) {
    let mut backoff = Duration::from_millis(100);
    let max_backoff = Duration::from_secs(30);
    let scope = config.scope.as_str();

    while !shutdown.load(Ordering::Relaxed) {
        match ipc_socket::connect_event_stream(socket_path).await {
            Ok(stream) => {
                internal::info(scope, "Connected to event socket");
                backoff = Duration::from_millis(100);
                process_events(EventStream::new(stream), logger, config, shutdown).await;

                if !shutdown.load(Ordering::Relaxed) {
                    internal::warn(scope, "Event socket disconnected, reconnecting...");
                }
            }
            Err(e) => {
                if shutdown.load(Ordering::Relaxed) {
                    break;
                }
                internal::error(scope, &format!("Failed to connect: {e}"));
                internal::error(scope, &format!("Retrying in {backoff:?}"));
            }
        }

        if !shutdown.load(Ordering::Relaxed) {
            time::sleep(backoff).await;
            backoff = (backoff * 2).min(max_backoff);
        }
    }

    internal::debug(scope, "Event listener stopped");
}

/// Inner loop for a single connection — returns when the socket drops so the outer loop can reconnect.
async fn process_events(
    mut events: EventStream,
    logger: &Logger,
    config: &HyprlandConfig,
    shutdown: &AtomicBool,
) {
    let scope = config.scope.as_str();
    let mut formatter = EventFormatter::new(config.human_readable);
    loop {
        if shutdown.load(Ordering::Relaxed) {
            break;
        }

        match time::timeout(Duration::from_secs(1), events.next_event()).await {
            Ok(Ok(Some(event))) => {
                let event = HyprlandEvent::from_sdk(&event);

                // User-configured blocklist — high-frequency events like activewindow can flood logs
                if config.ignore_events.iter().any(|e| e == &event.name) {
                    continue;
                }

                // Allowlist takes precedence over blocklist — if set, only these events pass through
                if let Some(ref filter) = config.event_filter
                    && !filter.iter().any(|f| f == &event.name)
                {
                    continue;
                }

                let level = resolve_level(&event.name, &config.event_levels);
                let scope = resolve_event_scope(config, &event);
                let msg = formatter.format(&event);
                formatter.observe(&event);
                logger.log(level, &scope, &msg);
            }
            Ok(Ok(None)) => break, // EOF — socket closed
            Ok(Err(e)) => {
                if !shutdown.load(Ordering::Relaxed) {
                    internal::warn(scope, &format!("Read error: {e}"));
                }
                break;
            }
            Err(_) => {
                // 1-second timeout ensures the shutdown flag is checked even when no events arrive
            }
        }
    }
}

fn resolve_event_scope(config: &HyprlandConfig, event: &HyprlandEvent) -> String {
    if let Some(scope) = config.event_scopes.get(&event.name) {
        return scope.clone();
    }

    if event.name == "custom" {
        return "hyprctl".to_string();
    }

    if let Some(scope) = extract_app_scope(event) {
        return scope;
    }

    if let Some(scope) = find_hypr_app_token(&event.data) {
        return scope;
    }

    "hyprland".to_string()
}

fn extract_app_scope(event: &HyprlandEvent) -> Option<String> {
    let candidate = match event.name.as_str() {
        "openwindow" => nth_csv_field(&event.data, 2),
        "activewindow" => nth_csv_field(&event.data, 0),
        _ => None,
    }?;

    normalize_scope_candidate(candidate)
}

fn normalize_scope_candidate(candidate: &str) -> Option<String> {
    let trimmed = candidate.trim();
    if trimmed.is_empty() {
        return None;
    }

    if is_monitor_name(trimmed) {
        return None;
    }

    if trimmed.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    if trimmed.len() >= 6 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }

    Some(trimmed.to_ascii_lowercase())
}

fn is_monitor_name(token: &str) -> bool {
    let upper = token.to_ascii_uppercase();
    upper.starts_with("DP-")
        || upper.starts_with("EDP-")
        || upper.starts_with("HDMI-")
        || upper.starts_with("DVI-")
        || upper.starts_with("VGA-")
        || upper.starts_with("WL-")
        || upper.starts_with("HEADLESS-")
}

fn find_hypr_app_token(data: &str) -> Option<String> {
    data.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
        .filter(|token| !token.is_empty())
        .find_map(|token| {
            normalize_scope_candidate(token).filter(|scope| scope.starts_with("hypr"))
        })
}

fn nth_csv_field(data: &str, n: usize) -> Option<&str> {
    data.split(',').nth(n)
}

/// Runs on a dedicated thread so the REPL stays responsive — returns None when
/// the socket can't be found (Hyprland not running) rather than blocking startup.
#[must_use]
pub fn start_listener(logger: Arc<Logger>, config: &HyprlandConfig) -> Option<EventListenerHandle> {
    let socket_dir = socket::resolve_socket_dir(config)?;
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = Arc::clone(&shutdown);
    let config = config.clone();
    let scope = config.scope.clone();

    let thread = match thread::Builder::new()
        .name("hyprland-listener".into())
        .spawn(move || {
            run_event_loop(&socket_dir, &logger, &config, &shutdown_clone);
        }) {
        Ok(handle) => handle,
        Err(e) => {
            internal::error(
                scope.as_str(),
                &format!("Failed to spawn listener thread: {e}"),
            );
            return None;
        }
    };

    Some(EventListenerHandle {
        shutdown,
        thread: Some(thread),
    })
}
