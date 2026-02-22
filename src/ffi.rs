//! C-ABI FFI bindings so C, C++, C#, and other languages can log through hyprlog
//! without linking against the Rust standard library directly.

#![allow(unsafe_code)]

use std::cell::RefCell;
use std::ffi::{CStr, c_char, c_int};
use std::path::Path;
use std::ptr;

use crate::config::Config;
use crate::fmt::IconSet;
use crate::internal;
use crate::level::Level;
use crate::logger::Logger;

/// Named constants so FFI callers avoid magic numbers in their log calls.
pub const HYPRLOG_LEVEL_TRACE: c_int = 0;
pub const HYPRLOG_LEVEL_DEBUG: c_int = 1;
pub const HYPRLOG_LEVEL_INFO: c_int = 2;
pub const HYPRLOG_LEVEL_WARN: c_int = 3;
pub const HYPRLOG_LEVEL_ERROR: c_int = 4;

/// Opaque pointer for C callers — hides the Rust Logger behind a stable ABI boundary.
pub struct HyprlogContext {
    logger: Logger,
    last_error: RefCell<Option<String>>,
}

impl HyprlogContext {
    fn set_error(&self, err: String) {
        *self.last_error.borrow_mut() = Some(err);
    }

    fn clear_error(&self) {
        *self.last_error.borrow_mut() = None;
    }
}

/// Maps the C-side integer constants back to the Rust enum for dispatch.
const fn level_from_int(level: c_int) -> Level {
    match level {
        0 => Level::Trace,
        1 => Level::Debug,
        3 => Level::Warn,
        4 => Level::Error,
        _ => Level::Info,
    }
}

/// Mirrors the CLI logger construction but scoped to the FFI caller's config.
fn build_logger(config: &Config) -> Logger {
    let mut builder = Logger::builder().level(config.parse_level());

    if config.terminal.enabled {
        builder = builder
            .terminal()
            .colors(config.terminal.colors)
            .icons(IconSet::from(config.parse_icon_type()))
            .structure(&config.terminal.structure)
            .done();
    }

    if config.file.enabled {
        builder = builder
            .file()
            .base_dir(&config.file.base_dir)
            .path_structure(&config.file.path_structure)
            .filename_structure(&config.file.filename_structure)
            .content_structure(&config.file.content_structure)
            .timestamp_format(&config.file.timestamp_format)
            .app_name(config.general.app_name.as_deref().unwrap_or("hyprlog"))
            .done();
    }

    builder.build()
}

// ============================================================================
// Initialization
// ============================================================================

/// Simplest entry point for C callers — loads the user's config or falls back to defaults.
///
/// Returns `NULL` on failure. Use `hyprlog_get_last_error` to retrieve error.
#[unsafe(no_mangle)]
pub extern "C" fn hyprlog_init() -> *mut HyprlogContext {
    internal::debug("FFI", "Logger initialized");
    let config = Config::load().unwrap_or_default();
    let logger = build_logger(&config);

    let ctx = Box::new(HyprlogContext {
        logger,
        last_error: RefCell::new(None),
    });

    Box::into_raw(ctx)
}

/// Allows C callers to point at a non-standard config file (e.g., for testing or embedding).
///
/// # Safety
/// `config_path` must be a valid null-terminated UTF-8 string or `NULL`.
/// If `NULL`, uses default config path.
///
/// Returns `NULL` on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hyprlog_init_with_config(
    config_path: *const c_char,
) -> *mut HyprlogContext {
    let config = if config_path.is_null() {
        Config::load().unwrap_or_default()
    } else {
        // SAFETY: config_path is non-null and caller guarantees valid UTF-8
        let Ok(path_str) = unsafe { CStr::from_ptr(config_path) }.to_str() else {
            internal::error("FFI", "Invalid UTF-8 in config_path");
            return ptr::null_mut();
        };
        internal::debug("FFI", &format!("Logger from {path_str}"));
        let Ok(c) = Config::load_from(Path::new(path_str)) else {
            return ptr::null_mut();
        };
        c
    };

    let logger = build_logger(&config);

    let ctx = Box::new(HyprlogContext {
        logger,
        last_error: RefCell::new(None),
    });

    Box::into_raw(ctx)
}

/// Uses the global config but routes file logs into an app-specific subdirectory.
///
/// # Safety
/// `app_name` must be a valid null-terminated UTF-8 string.
///
/// Returns `NULL` on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hyprlog_init_with_app(app_name: *const c_char) -> *mut HyprlogContext {
    if app_name.is_null() {
        return ptr::null_mut();
    }

    // SAFETY: app_name is non-null and caller guarantees valid UTF-8
    let Ok(app_str) = unsafe { CStr::from_ptr(app_name) }.to_str() else {
        internal::error("FFI", "Invalid UTF-8 in app_name");
        return ptr::null_mut();
    };

    internal::debug("FFI", &format!("Logger for app {app_str}"));
    let logger = Logger::from_config(app_str);

    let ctx = Box::new(HyprlogContext {
        logger,
        last_error: RefCell::new(None),
    });

    Box::into_raw(ctx)
}

/// Bypasses config entirely for callers that just need quick terminal logging.
///
/// # Arguments
/// * `level` - Minimum log level (0=trace, 1=debug, 2=info, 3=warn, 4=error)
/// * `colors` - Enable ANSI colors (1=true, 0=false)
#[unsafe(no_mangle)]
pub extern "C" fn hyprlog_init_simple(level: c_int, colors: c_int) -> *mut HyprlogContext {
    let logger = Logger::builder()
        .level(level_from_int(level))
        .terminal()
        .colors(colors != 0)
        .done()
        .build();

    let ctx = Box::new(HyprlogContext {
        logger,
        last_error: RefCell::new(None),
    });

    Box::into_raw(ctx)
}

/// Releases the heap-allocated context — must be called to avoid leaking memory.
///
/// # Safety
/// `ctx` must be a valid pointer returned by `hyprlog_init*` or `NULL`.
/// After this call, `ctx` is invalid and must not be used.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hyprlog_free(ctx: *mut HyprlogContext) {
    if !ctx.is_null() {
        // SAFETY: ctx is non-null and was created by Box::into_raw
        drop(unsafe { Box::from_raw(ctx) });
    }
}

// ============================================================================
// Logging
// ============================================================================

/// Core logging function — all level-specific wrappers delegate here.
///
/// # Safety
/// - `ctx` must be a valid context pointer
/// - `scope` and `msg` must be valid null-terminated UTF-8 strings
///
/// # Arguments
/// * `ctx` - Logger context
/// * `level` - Log level (0=trace, 1=debug, 2=info, 3=warn, 4=error)
/// * `scope` - Log scope/category (e.g., "NET", "DB", "MAIN")
/// * `msg` - Log message
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hyprlog_log(
    ctx: *mut HyprlogContext,
    level: c_int,
    scope: *const c_char,
    msg: *const c_char,
) {
    if ctx.is_null() || scope.is_null() || msg.is_null() {
        return;
    }

    // SAFETY: ctx is non-null and valid
    let context = unsafe { &*ctx };
    context.clear_error();

    // SAFETY: scope is non-null and caller guarantees valid UTF-8
    let Ok(scope_str) = unsafe { CStr::from_ptr(scope) }.to_str() else {
        return;
    };

    // SAFETY: msg is non-null and caller guarantees valid UTF-8
    let Ok(msg_str) = unsafe { CStr::from_ptr(msg) }.to_str() else {
        return;
    };

    context
        .logger
        .log(level_from_int(level), scope_str, msg_str);
}

/// Convenience wrapper — avoids passing the level constant for every trace call.
///
/// # Safety
/// See `hyprlog_log`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hyprlog_trace(
    ctx: *mut HyprlogContext,
    scope: *const c_char,
    msg: *const c_char,
) {
    // SAFETY: delegating to hyprlog_log with same safety requirements
    unsafe { hyprlog_log(ctx, HYPRLOG_LEVEL_TRACE, scope, msg) };
}

/// Convenience wrapper — avoids passing the level constant for every debug call.
///
/// # Safety
/// See `hyprlog_log`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hyprlog_debug(
    ctx: *mut HyprlogContext,
    scope: *const c_char,
    msg: *const c_char,
) {
    // SAFETY: delegating to hyprlog_log with same safety requirements
    unsafe { hyprlog_log(ctx, HYPRLOG_LEVEL_DEBUG, scope, msg) };
}

/// Convenience wrapper — avoids passing the level constant for every info call.
///
/// # Safety
/// See `hyprlog_log`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hyprlog_info(
    ctx: *mut HyprlogContext,
    scope: *const c_char,
    msg: *const c_char,
) {
    // SAFETY: delegating to hyprlog_log with same safety requirements
    unsafe { hyprlog_log(ctx, HYPRLOG_LEVEL_INFO, scope, msg) };
}

/// Convenience wrapper — avoids passing the level constant for every warn call.
///
/// # Safety
/// See `hyprlog_log`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hyprlog_warn(
    ctx: *mut HyprlogContext,
    scope: *const c_char,
    msg: *const c_char,
) {
    // SAFETY: delegating to hyprlog_log with same safety requirements
    unsafe { hyprlog_log(ctx, HYPRLOG_LEVEL_WARN, scope, msg) };
}

/// Convenience wrapper — avoids passing the level constant for every error call.
///
/// # Safety
/// See `hyprlog_log`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hyprlog_error(
    ctx: *mut HyprlogContext,
    scope: *const c_char,
    msg: *const c_char,
) {
    // SAFETY: delegating to hyprlog_log with same safety requirements
    unsafe { hyprlog_log(ctx, HYPRLOG_LEVEL_ERROR, scope, msg) };
}

// ============================================================================
// Error Handling
// ============================================================================

/// Copies the last error into a caller-owned buffer — the C-side error reporting pattern.
///
/// # Safety
/// - `ctx` must be a valid context pointer
/// - `buffer` must point to a writable buffer of at least `len` bytes
///
/// # Returns
/// - Length of error message on success
/// - 0 if no error
/// - -1 on invalid arguments
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hyprlog_get_last_error(
    ctx: *mut HyprlogContext,
    buffer: *mut c_char,
    len: usize,
) -> c_int {
    if ctx.is_null() || buffer.is_null() || len == 0 {
        return -1;
    }

    // SAFETY: ctx is non-null and valid
    let context = unsafe { &*ctx };
    let borrow = context.last_error.borrow();

    if let Some(msg) = &*borrow {
        let bytes = msg.as_bytes();
        if bytes.len() >= len {
            return -1;
        }

        // SAFETY: buffer has at least len bytes and bytes.len() < len
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), buffer.cast::<u8>(), bytes.len());
            *buffer.add(bytes.len()) = 0;
        }

        c_int::try_from(bytes.len()).unwrap_or(-1)
    } else {
        // SAFETY: buffer is non-null and has at least 1 byte
        unsafe { *buffer = 0 };
        0
    }
}

/// Forces all buffered output to disk/terminal — call before program exit to avoid lost logs.
///
/// # Safety
/// `ctx` must be a valid context pointer.
///
/// # Returns
/// 0 on success, non-zero on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hyprlog_flush(ctx: *mut HyprlogContext) -> c_int {
    if ctx.is_null() {
        return -1;
    }

    // SAFETY: ctx is non-null and valid
    let context = unsafe { &*ctx };
    context.clear_error();

    match context.logger.flush() {
        Ok(()) => 0,
        Err(e) => {
            context.set_error(e.to_string());
            1
        }
    }
}
