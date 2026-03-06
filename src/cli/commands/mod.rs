//! CLI command implementations.

mod cleanup;
mod json;
mod log;
mod preset;
mod send;
mod stats;
mod themes;

#[cfg(feature = "hyprland")]
mod hyprland;

#[cfg(feature = "rserver")]
mod server;

pub use cleanup::cmd_cleanup;
pub use json::cmd_json;
pub use log::{cmd_log, cmd_log_shorthand};
pub use preset::{cmd_preset, cmd_presets};
pub use send::cmd_send;
pub use stats::cmd_stats;
pub use themes::cmd_themes;

#[cfg(feature = "hyprland")]
pub use hyprland::cmd_watch;

#[cfg(feature = "rserver")]
pub use server::cmd_server;
