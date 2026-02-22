//! The REPL's background listener is invisible — `watch` gives users a dedicated
//! foreground mode to see events in real time for debugging compositor issues.

use crate::config::Config;
use crate::hyprland::{listener, socket};
use crate::internal;
use crate::level::Level;
use crate::logger::Logger;
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Global static because POSIX signal handlers can't capture closures —
/// the handler must store to a fixed address. `AtomicBool::store` with
/// `Relaxed` ordering is async-signal-safe on all platforms Rust supports.
static SHUTDOWN: AtomicBool = AtomicBool::new(false);

/// The event loop checks an `Arc<AtomicBool>` — bridging the global static to an Arc
/// decouples the signal plumbing from the listener's shutdown interface.
fn install_shutdown_handler() -> Arc<AtomicBool> {
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = Arc::clone(&shutdown);

    // Previous invocations may have left this set — reset so the new session starts clean
    SHUTDOWN.store(false, Ordering::Relaxed);

    // Watcher thread bridges static→Arc because signal handlers can't access heap-allocated Arcs
    std::thread::Builder::new()
        .name("signal-watcher".into())
        .spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(100));
                if SHUTDOWN.load(Ordering::Relaxed) {
                    shutdown_clone.store(true, Ordering::Relaxed);
                    break;
                }
            }
        })
        .ok();

    // POSIX signal handler — only atomic stores are safe inside signal context
    // SAFETY: The handler only performs an atomic store, which is async-signal-safe.
    #[cfg(unix)]
    unsafe {
        register_sigint_handler();
    }

    shutdown
}

/// Minimal handler body — anything beyond an atomic store risks undefined behavior in signal context.
#[cfg(unix)]
extern "C" fn sigint_handler(_sig: std::ffi::c_int) {
    SHUTDOWN.store(true, Ordering::Relaxed);
}

/// Raw `signal()` call instead of a crate because we only need SIGINT and
/// adding a dependency for one syscall wrapper isn't worth the cost.
///
/// # Safety
/// The handler only performs an async-signal-safe atomic store.
#[cfg(unix)]
unsafe fn register_sigint_handler() {
    // Hardcoded SIGINT=2 avoids depending on libc just for a constant
    unsafe extern "C" {
        fn signal(sig: std::ffi::c_int, handler: extern "C" fn(std::ffi::c_int)) -> usize;
    }
    unsafe { signal(2, sigint_handler) };
}

/// Foreground event streaming — blocks until Ctrl+C so the user sees events
/// in real time. Useful for debugging window rules, keybinds, and compositor behavior.
#[must_use]
pub fn cmd_watch(args: &[&str], config: &Config, logger: &Logger) -> ExitCode {
    let mut hyprland_config = config.hyprland.clone();

    // Allowlist filter — 43 event types flood the terminal, users usually care about 2-3
    if let Some(idx) = args.iter().position(|&a| a == "--events")
        && let Some(&filter) = args.get(idx + 1)
    {
        let allowed: Vec<String> = filter.split(',').map(|s| s.trim().to_string()).collect();
        hyprland_config.event_filter = Some(allowed);
    }

    // Min-level filter — hides Debug-level events (focus changes etc.) that dominate the stream
    if let Some(idx) = args.iter().position(|&a| a == "--min-level")
        && let Some(&level_str) = args.get(idx + 1)
    {
        if let Ok(min_level) = level_str.parse::<Level>() {
            // Events below min-level get added to the ignore list rather than changing the level map
            let defaults = crate::hyprland::level_map::default_level_map();
            for (&event_name, &default_level) in &defaults {
                if default_level < min_level
                    && !hyprland_config.event_levels.contains_key(event_name)
                {
                    hyprland_config.ignore_events.push(event_name.to_string());
                }
            }
        } else {
            internal::error(
                hyprland_config.scope.as_str(),
                &format!("Invalid level: {level_str}"),
            );
            return ExitCode::FAILURE;
        }
    }

    let Some(socket_dir) = socket::resolve_socket_dir(&hyprland_config) else {
        return ExitCode::FAILURE;
    };

    let shutdown = install_shutdown_handler();

    logger.print(
        hyprland_config.scope.as_str(),
        "Listening for Hyprland events... (Ctrl+C to stop)",
    );
    listener::run_event_loop(&socket_dir, logger, &hyprland_config, &shutdown);
    ExitCode::SUCCESS
}
