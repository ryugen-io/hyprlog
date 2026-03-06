//! C-ABI FFI bindings for hyprslog.
//!
//! Provides a stable C interface for logging from C, C++, C#, and other languages.

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

/// Log level constants for FFI.
pub const HYPRSLOG_LEVEL_TRACE: c_int = 0;
pub const HYPRSLOG_LEVEL_DEBUG: c_int = 1;
pub const HYPRSLOG_LEVEL_INFO: c_int = 2;
pub const HYPRSLOG_LEVEL_WARN: c_int = 3;
pub const HYPRSLOG_LEVEL_ERROR: c_int = 4;

/// Opaque context holding the logger and error state.
pub struct HyprslogContext {
    logger: Logger,
    last_error: RefCell<Option<String>>,
}

impl HyprslogContext {
    fn set_error(&self, err: String) {
        *self.last_error.borrow_mut() = Some(err);
    }

    fn clear_error(&self) {
        *self.last_error.borrow_mut() = None;
    }
}

/// Converts a C int to a Level enum.
const fn level_from_int(level: c_int) -> Level {
    match level {
        0 => Level::Trace,
        1 => Level::Debug,
        3 => Level::Warn,
        4 => Level::Error,
        _ => Level::Info,
    }
}

/// Builds a logger from a Config.
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
            .app_name(config.general.app_name.as_deref().unwrap_or("hyprslog"))
            .done();
    }

    builder.build()
}

// ============================================================================
// Initialization
// ============================================================================

/// Creates a new hyprslog context with default configuration.
///
/// Loads config from `~/.config/hypr/hyprs/log.conf` if present,
/// otherwise uses defaults.
///
/// Returns `NULL` on failure. Use `hyprslog_get_last_error` to retrieve error.
#[unsafe(no_mangle)]
pub extern "C" fn hyprslog_init() -> *mut HyprslogContext {
    internal::debug("FFI", "Logger initialized");
    let config = Config::load().unwrap_or_default();
    let logger = build_logger(&config);

    let ctx = Box::new(HyprslogContext {
        logger,
        last_error: RefCell::new(None),
    });

    Box::into_raw(ctx)
}

/// Creates a new hyprslog context with configuration from the specified path.
///
/// # Safety
/// `config_path` must be a valid null-terminated UTF-8 string or `NULL`.
/// If `NULL`, uses default config path.
///
/// Returns `NULL` on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hyprslog_init_with_config(
    config_path: *const c_char,
) -> *mut HyprslogContext {
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

    let ctx = Box::new(HyprslogContext {
        logger,
        last_error: RefCell::new(None),
    });

    Box::into_raw(ctx)
}

/// Creates a logger with default config but custom app name.
///
/// Loads config from `~/.config/hypr/hyprs/log.conf` if present,
/// but uses the provided `app_name` for file logging paths.
///
/// # Safety
/// `app_name` must be a valid null-terminated UTF-8 string.
///
/// Returns `NULL` on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hyprslog_init_with_app(app_name: *const c_char) -> *mut HyprslogContext {
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

    let ctx = Box::new(HyprslogContext {
        logger,
        last_error: RefCell::new(None),
    });

    Box::into_raw(ctx)
}

/// Creates a minimal logger with only terminal output (no config file).
///
/// Useful for quick setup without configuration.
///
/// # Arguments
/// * `level` - Minimum log level (0=trace, 1=debug, 2=info, 3=warn, 4=error)
/// * `colors` - Enable ANSI colors (1=true, 0=false)
#[unsafe(no_mangle)]
pub extern "C" fn hyprslog_init_simple(level: c_int, colors: c_int) -> *mut HyprslogContext {
    let logger = Logger::builder()
        .level(level_from_int(level))
        .terminal()
        .colors(colors != 0)
        .done()
        .build();

    let ctx = Box::new(HyprslogContext {
        logger,
        last_error: RefCell::new(None),
    });

    Box::into_raw(ctx)
}

/// Frees a hyprslog context.
///
/// # Safety
/// `ctx` must be a valid pointer returned by `hyprslog_init*` or `NULL`.
/// After this call, `ctx` is invalid and must not be used.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hyprslog_free(ctx: *mut HyprslogContext) {
    if !ctx.is_null() {
        // SAFETY: ctx is non-null and was created by Box::into_raw
        drop(unsafe { Box::from_raw(ctx) });
    }
}

// ============================================================================
// Logging
// ============================================================================

/// Logs a message at the specified level.
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
pub unsafe extern "C" fn hyprslog_log(
    ctx: *mut HyprslogContext,
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

/// Logs a trace message.
///
/// # Safety
/// See `hyprslog_log`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hyprslog_trace(
    ctx: *mut HyprslogContext,
    scope: *const c_char,
    msg: *const c_char,
) {
    // SAFETY: delegating to hyprslog_log with same safety requirements
    unsafe { hyprslog_log(ctx, HYPRSLOG_LEVEL_TRACE, scope, msg) };
}

/// Logs a debug message.
///
/// # Safety
/// See `hyprslog_log`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hyprslog_debug(
    ctx: *mut HyprslogContext,
    scope: *const c_char,
    msg: *const c_char,
) {
    // SAFETY: delegating to hyprslog_log with same safety requirements
    unsafe { hyprslog_log(ctx, HYPRSLOG_LEVEL_DEBUG, scope, msg) };
}

/// Logs an info message.
///
/// # Safety
/// See `hyprslog_log`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hyprslog_info(
    ctx: *mut HyprslogContext,
    scope: *const c_char,
    msg: *const c_char,
) {
    // SAFETY: delegating to hyprslog_log with same safety requirements
    unsafe { hyprslog_log(ctx, HYPRSLOG_LEVEL_INFO, scope, msg) };
}

/// Logs a warning message.
///
/// # Safety
/// See `hyprslog_log`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hyprslog_warn(
    ctx: *mut HyprslogContext,
    scope: *const c_char,
    msg: *const c_char,
) {
    // SAFETY: delegating to hyprslog_log with same safety requirements
    unsafe { hyprslog_log(ctx, HYPRSLOG_LEVEL_WARN, scope, msg) };
}

/// Logs an error message.
///
/// # Safety
/// See `hyprslog_log`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hyprslog_error(
    ctx: *mut HyprslogContext,
    scope: *const c_char,
    msg: *const c_char,
) {
    // SAFETY: delegating to hyprslog_log with same safety requirements
    unsafe { hyprslog_log(ctx, HYPRSLOG_LEVEL_ERROR, scope, msg) };
}

// ============================================================================
// Error Handling
// ============================================================================

/// Retrieves the last error message.
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
pub unsafe extern "C" fn hyprslog_get_last_error(
    ctx: *mut HyprslogContext,
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

/// Flushes all log outputs.
///
/// # Safety
/// `ctx` must be a valid context pointer.
///
/// # Returns
/// 0 on success, non-zero on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hyprslog_flush(ctx: *mut HyprslogContext) -> c_int {
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
