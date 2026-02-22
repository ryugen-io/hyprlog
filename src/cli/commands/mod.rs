//! Each subcommand lives in its own file — keeps the match arm in main thin
//! and lets each handler own its argument validation and error reporting.

mod cleanup;
mod json;
mod log;
mod preset;
mod stats;
mod themes;

#[cfg(feature = "hyprland")]
mod hyprland;

pub use cleanup::cmd_cleanup;
pub use json::cmd_json;
pub use log::{cmd_log, cmd_log_shorthand};
pub use preset::{cmd_preset, cmd_presets};
pub use stats::cmd_stats;
pub use themes::cmd_themes;

#[cfg(feature = "hyprland")]
pub use hyprland::cmd_watch;
