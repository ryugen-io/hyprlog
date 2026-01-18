//! `hl_common` - Shared CLI utilities for hyprlog.
//!
//! This crate provides shared functionality between `hl_cli` and `hl_shell`:
//! - Preset execution from config
//! - Shared argument types and validation

pub mod args;
pub mod preset;

pub use args::LogArgs;
pub use preset::PresetRunner;
