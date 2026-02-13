//! Hyprland socket2 event listener.

use super::event::HyprlandEvent;
use super::level_map::resolve_level;
use super::socket;
use crate::config::HyprlandConfig;
use crate::internal;
use crate::logger::Logger;
use std::io::{self, BufRead};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Handle to a running event listener thread.
///
/// Drop this handle to signal shutdown and wait for the listener to stop.
pub struct EventListenerHandle {
    shutdown: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl EventListenerHandle {
    /// Signals the listener to stop.
    pub fn stop(&self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }

    /// Signals stop and waits for the listener thread to finish.
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

/// Runs the event listener loop on the current thread (blocking).
///
/// Connects to socket2, reads events line by line, and routes them through the
/// logger. Reconnects with exponential backoff on disconnect.
///
/// Respects `config.ignore_events` to skip unwanted events, and applies
/// per-event level mapping via `config.event_levels`.
pub fn run_event_loop(
    socket_dir: &std::path::Path,
    logger: &Logger,
    config: &HyprlandConfig,
    shutdown: &AtomicBool,
) {
    let mut backoff = Duration::from_millis(100);
    let max_backoff = Duration::from_secs(30);

    while !shutdown.load(Ordering::Relaxed) {
        if let Some(reader) = socket::connect_event_stream(socket_dir) {
            internal::info("HYPRLAND", "Connected to event socket");
            backoff = Duration::from_millis(100);
            process_events(reader, logger, config, shutdown);

            if !shutdown.load(Ordering::Relaxed) {
                internal::warn("HYPRLAND", "Event socket disconnected, reconnecting...");
            }
        } else {
            if shutdown.load(Ordering::Relaxed) {
                break;
            }
            internal::error("HYPRLAND", &format!("Retrying in {backoff:?}"));
        }

        if !shutdown.load(Ordering::Relaxed) {
            thread::sleep(backoff);
            backoff = (backoff * 2).min(max_backoff);
        }
    }

    internal::debug("HYPRLAND", "Event listener stopped");
}

/// Processes events from a connected socket2 stream.
fn process_events(
    mut reader: io::BufReader<std::os::unix::net::UnixStream>,
    logger: &Logger,
    config: &HyprlandConfig,
    shutdown: &AtomicBool,
) {
    let mut line = String::new();

    loop {
        if shutdown.load(Ordering::Relaxed) {
            break;
        }

        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break, // EOF — socket closed
            Ok(_) => {
                if let Some(event) = HyprlandEvent::parse(&line) {
                    // Skip ignored events
                    if config.ignore_events.iter().any(|e| e == &event.name) {
                        continue;
                    }

                    // If an allowlist filter is set, skip events not in it
                    if let Some(ref filter) = config.event_filter {
                        if !filter.iter().any(|f| f == &event.name) {
                            continue;
                        }
                    }

                    let level = resolve_level(&event.name, &config.event_levels);
                    logger.log(level, &config.scope, &event.format_message());
                }
            }
            Err(ref e)
                if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut =>
            {
                // Read timeout — loop will check shutdown flag
            }
            Err(e) => {
                if !shutdown.load(Ordering::Relaxed) {
                    internal::warn("HYPRLAND", &format!("Read error: {e}"));
                }
                break;
            }
        }
    }
}

/// Starts the event listener in a background thread.
///
/// Returns a handle that can be used to stop the listener, or `None` if the
/// socket directory cannot be resolved or the thread fails to spawn.
#[must_use]
pub fn start_listener(logger: Arc<Logger>, config: &HyprlandConfig) -> Option<EventListenerHandle> {
    let socket_dir = socket::resolve_socket_dir(config)?;
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = Arc::clone(&shutdown);
    let config = config.clone();

    let thread = match thread::Builder::new()
        .name("hyprland-listener".into())
        .spawn(move || {
            run_event_loop(&socket_dir, &logger, &config, &shutdown_clone);
        }) {
        Ok(handle) => handle,
        Err(e) => {
            internal::error("HYPRLAND", &format!("Failed to spawn listener thread: {e}"));
            return None;
        }
    };

    Some(EventListenerHandle {
        shutdown,
        thread: Some(thread),
    })
}
