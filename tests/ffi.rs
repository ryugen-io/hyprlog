//! Tests for FFI functionality.

#![cfg(feature = "ffi")]

use hyprs_log::{
    HYPRSLOG_LEVEL_DEBUG, HYPRSLOG_LEVEL_ERROR, HYPRSLOG_LEVEL_INFO, HYPRSLOG_LEVEL_TRACE,
    HYPRSLOG_LEVEL_WARN, hyprslog_free, hyprslog_init, hyprslog_init_simple,
};
use std::ptr;

#[test]
fn test_init_free() {
    let ctx = hyprslog_init();
    assert!(!ctx.is_null());
    unsafe {
        hyprslog_free(ctx);
    }
}

#[test]
fn test_init_simple() {
    let ctx = hyprslog_init_simple(HYPRSLOG_LEVEL_DEBUG, 0);
    assert!(!ctx.is_null());
    unsafe {
        hyprslog_free(ctx);
    }
}

#[test]
fn test_free_null() {
    unsafe {
        hyprslog_free(ptr::null_mut());
    }
}

#[test]
fn test_level_constants() {
    assert_eq!(HYPRSLOG_LEVEL_TRACE, 0);
    assert_eq!(HYPRSLOG_LEVEL_DEBUG, 1);
    assert_eq!(HYPRSLOG_LEVEL_INFO, 2);
    assert_eq!(HYPRSLOG_LEVEL_WARN, 3);
    assert_eq!(HYPRSLOG_LEVEL_ERROR, 4);
}
