//! Hyprland event listener CLI command.

use crate::config::Config;
use crate::hyprland::{listener, socket};
use crate::internal;
use crate::level::Level;
use crate::logger::Logger;
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Global shutdown flag for signal handling.
///
/// Set by the SIGINT handler registered via `install_shutdown_handler`.
/// Signal-safe because `AtomicBool::store` with `Relaxed` ordering is
/// async-signal-safe on all platforms Rust supports.
static SHUTDOWN: AtomicBool = AtomicBool::new(false);

/// Installs a Ctrl+C handler that sets a shutdown flag.
///
/// Returns an `Arc<AtomicBool>` that mirrors the global static, allowing
/// callers to poll without accessing the static directly.
fn install_shutdown_handler() -> Arc<AtomicBool> {
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = Arc::clone(&shutdown);

    // Reset the static flag
    SHUTDOWN.store(false, Ordering::Relaxed);

    // Watcher thread: polls the static flag and propagates to the Arc
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

    // Register POSIX signal handler
    // SAFETY: The handler only performs an atomic store, which is async-signal-safe.
    #[cfg(unix)]
    unsafe {
        register_sigint_handler();
    }

    shutdown
}

/// Signal handler function — must be async-signal-safe.
#[cfg(unix)]
extern "C" fn sigint_handler(_sig: std::ffi::c_int) {
    SHUTDOWN.store(true, Ordering::Relaxed);
}

/// Registers SIGINT handler via libc's `signal()`.
///
/// # Safety
/// Calls POSIX `signal()`. The handler only performs an atomic store.
#[cfg(unix)]
unsafe fn register_sigint_handler() {
    // SIGINT = 2 on all POSIX platforms
    unsafe extern "C" {
        fn signal(sig: std::ffi::c_int, handler: extern "C" fn(std::ffi::c_int)) -> usize;
    }
    unsafe { signal(2, sigint_handler) };
}

/// Handles `hyprlog watch [--events <filter>] [--min-level <level>]`.
///
/// Connects to Hyprland's event socket and streams events through the logger.
/// Blocks until Ctrl+C.
#[must_use]
pub fn cmd_watch(args: &[&str], config: &Config, logger: &Logger) -> ExitCode {
    let mut hyprland_config = config.hyprland.clone();

    // Parse --events filter (comma-separated allowlist)
    if let Some(idx) = args.iter().position(|&a| a == "--events") {
        if let Some(&filter) = args.get(idx + 1) {
            let allowed: Vec<String> = filter.split(',').map(|s| s.trim().to_string()).collect();
            hyprland_config.event_filter = Some(allowed);
        }
    }

    // Parse --min-level filter
    if let Some(idx) = args.iter().position(|&a| a == "--min-level") {
        if let Some(&level_str) = args.get(idx + 1) {
            if let Ok(min_level) = level_str.parse::<Level>() {
                // Add events below this level to the ignore list
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
