//! `hl_common` - Shared CLI utilities for hyprlog.
//!
//! This crate provides shared functionality between `hl_cli` and `hl_shell`:
//! - Preset execution from config
//! - Output formatting for stats and cleanup results
//! - Shared argument types and validation

pub mod args;
pub mod output;
pub mod preset;

pub use args::LogArgs;
pub use output::OutputFormatter;
pub use preset::PresetRunner;
