//! Tests for FFI functionality.

#![cfg(feature = "ffi")]

use hyprlog::{
    HYPRLOG_LEVEL_DEBUG, HYPRLOG_LEVEL_ERROR, HYPRLOG_LEVEL_INFO, HYPRLOG_LEVEL_TRACE,
    HYPRLOG_LEVEL_WARN, hyprlog_free, hyprlog_init, hyprlog_init_simple,
};
use std::ptr;

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
fn test_level_constants() {
    assert_eq!(HYPRLOG_LEVEL_TRACE, 0);
    assert_eq!(HYPRLOG_LEVEL_DEBUG, 1);
    assert_eq!(HYPRLOG_LEVEL_INFO, 2);
    assert_eq!(HYPRLOG_LEVEL_WARN, 3);
    assert_eq!(HYPRLOG_LEVEL_ERROR, 4);
}
