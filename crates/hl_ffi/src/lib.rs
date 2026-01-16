//! C-ABI FFI bindings for hyprlog.
//!
//! Provides a stable C interface for logging from C, C++, C#, and other languages.

use std::cell::RefCell;
use std::ffi::{c_char, c_int, CStr};
use std::path::Path;
use std::ptr;

use hl_core::{Config, IconSet, Level, Logger};

/// Log level constants for FFI.
pub const HYPRLOG_LEVEL_TRACE: c_int = 0;
pub const HYPRLOG_LEVEL_DEBUG: c_int = 1;
pub const HYPRLOG_LEVEL_INFO: c_int = 2;
pub const HYPRLOG_LEVEL_WARN: c_int = 3;
pub const HYPRLOG_LEVEL_ERROR: c_int = 4;

/// Opaque context holding the logger and error state.
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
            .app_name(&config.general.app_name)
            .done();
    }

    builder.build()
}

// ============================================================================
// Initialization
// ============================================================================

/// Creates a new hyprlog context with default configuration.
///
/// Loads config from `~/.config/hypr/hyprlog.conf` if present,
/// otherwise uses defaults.
///
/// Returns `NULL` on failure. Use `hyprlog_get_last_error` to retrieve error.
#[no_mangle]
pub extern "C" fn hyprlog_init() -> *mut HyprlogContext {
    let config = Config::load().unwrap_or_default();
    let logger = build_logger(&config);

    let ctx = Box::new(HyprlogContext {
        logger,
        last_error: RefCell::new(None),
    });

    Box::into_raw(ctx)
}

/// Creates a new hyprlog context with configuration from the specified path.
///
/// # Safety
/// `config_path` must be a valid null-terminated UTF-8 string or `NULL`.
/// If `NULL`, uses default config path.
///
/// Returns `NULL` on failure.
#[no_mangle]
pub unsafe extern "C" fn hyprlog_init_with_config(
    config_path: *const c_char,
) -> *mut HyprlogContext {
    let config = if config_path.is_null() {
        Config::load().unwrap_or_default()
    } else {
        let Ok(path_str) = CStr::from_ptr(config_path).to_str() else {
            return ptr::null_mut();
        };
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

/// Creates a logger with default config but custom app name.
///
/// Loads config from `~/.config/hypr/hyprlog.conf` if present,
/// but uses the provided `app_name` for file logging paths.
///
/// # Safety
/// `app_name` must be a valid null-terminated UTF-8 string.
///
/// Returns `NULL` on failure.
#[no_mangle]
pub unsafe extern "C" fn hyprlog_init_with_app(app_name: *const c_char) -> *mut HyprlogContext {
    if app_name.is_null() {
        return ptr::null_mut();
    }

    let Ok(app_str) = CStr::from_ptr(app_name).to_str() else {
        return ptr::null_mut();
    };

    let logger = Logger::from_config(app_str);

    let ctx = Box::new(HyprlogContext {
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
#[no_mangle]
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

/// Frees a hyprlog context.
///
/// # Safety
/// `ctx` must be a valid pointer returned by `hyprlog_init*` or `NULL`.
/// After this call, `ctx` is invalid and must not be used.
#[no_mangle]
pub unsafe extern "C" fn hyprlog_free(ctx: *mut HyprlogContext) {
    if !ctx.is_null() {
        drop(Box::from_raw(ctx));
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
#[no_mangle]
pub unsafe extern "C" fn hyprlog_log(
    ctx: *mut HyprlogContext,
    level: c_int,
    scope: *const c_char,
    msg: *const c_char,
) {
    if ctx.is_null() || scope.is_null() || msg.is_null() {
        return;
    }

    let context = &*ctx;
    context.clear_error();

    let Ok(scope_str) = CStr::from_ptr(scope).to_str() else {
        return;
    };

    let Ok(msg_str) = CStr::from_ptr(msg).to_str() else {
        return;
    };

    context
        .logger
        .log(level_from_int(level), scope_str, msg_str);
}

/// Logs a trace message.
///
/// # Safety
/// See `hyprlog_log`.
#[no_mangle]
pub unsafe extern "C" fn hyprlog_trace(
    ctx: *mut HyprlogContext,
    scope: *const c_char,
    msg: *const c_char,
) {
    hyprlog_log(ctx, HYPRLOG_LEVEL_TRACE, scope, msg);
}

/// Logs a debug message.
///
/// # Safety
/// See `hyprlog_log`.
#[no_mangle]
pub unsafe extern "C" fn hyprlog_debug(
    ctx: *mut HyprlogContext,
    scope: *const c_char,
    msg: *const c_char,
) {
    hyprlog_log(ctx, HYPRLOG_LEVEL_DEBUG, scope, msg);
}

/// Logs an info message.
///
/// # Safety
/// See `hyprlog_log`.
#[no_mangle]
pub unsafe extern "C" fn hyprlog_info(
    ctx: *mut HyprlogContext,
    scope: *const c_char,
    msg: *const c_char,
) {
    hyprlog_log(ctx, HYPRLOG_LEVEL_INFO, scope, msg);
}

/// Logs a warning message.
///
/// # Safety
/// See `hyprlog_log`.
#[no_mangle]
pub unsafe extern "C" fn hyprlog_warn(
    ctx: *mut HyprlogContext,
    scope: *const c_char,
    msg: *const c_char,
) {
    hyprlog_log(ctx, HYPRLOG_LEVEL_WARN, scope, msg);
}

/// Logs an error message.
///
/// # Safety
/// See `hyprlog_log`.
#[no_mangle]
pub unsafe extern "C" fn hyprlog_error(
    ctx: *mut HyprlogContext,
    scope: *const c_char,
    msg: *const c_char,
) {
    hyprlog_log(ctx, HYPRLOG_LEVEL_ERROR, scope, msg);
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
#[no_mangle]
pub unsafe extern "C" fn hyprlog_get_last_error(
    ctx: *mut HyprlogContext,
    buffer: *mut c_char,
    len: usize,
) -> c_int {
    if ctx.is_null() || buffer.is_null() || len == 0 {
        return -1;
    }

    let context = &*ctx;
    let borrow = context.last_error.borrow();

    if let Some(msg) = &*borrow {
        let bytes = msg.as_bytes();
        if bytes.len() >= len {
            return -1;
        }

        ptr::copy_nonoverlapping(bytes.as_ptr(), buffer.cast::<u8>(), bytes.len());
        *buffer.add(bytes.len()) = 0;

        c_int::try_from(bytes.len()).unwrap_or(-1)
    } else {
        *buffer = 0;
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
#[no_mangle]
pub unsafe extern "C" fn hyprlog_flush(ctx: *mut HyprlogContext) -> c_int {
    if ctx.is_null() {
        return -1;
    }

    let context = &*ctx;
    context.clear_error();

    match context.logger.flush() {
        Ok(()) => 0,
        Err(e) => {
            context.set_error(e.to_string());
            1
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_free() {
        let ctx = hyprlog_init();
        assert!(!ctx.is_null());
        unsafe {
            hyprlog_free(ctx);
        }
    }

    #[test]
    fn test_init_simple() {
        let ctx = hyprlog_init_simple(HYPRLOG_LEVEL_DEBUG, 0);
        assert!(!ctx.is_null());
        unsafe {
            hyprlog_free(ctx);
        }
    }

    #[test]
    fn test_free_null() {
        unsafe {
            hyprlog_free(ptr::null_mut());
        }
    }

    #[test]
    fn test_level_from_int() {
        assert_eq!(level_from_int(0), Level::Trace);
        assert_eq!(level_from_int(1), Level::Debug);
        assert_eq!(level_from_int(2), Level::Info);
        assert_eq!(level_from_int(3), Level::Warn);
        assert_eq!(level_from_int(4), Level::Error);
        assert_eq!(level_from_int(99), Level::Info); // default
    }
}
