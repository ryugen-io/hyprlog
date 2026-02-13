//! Hyprland IPC integration.
//!
//! Listens to Hyprland's event socket (socket2) for compositor events
//! and routes them through the logger.

pub mod event;
pub mod level_map;
pub mod listener;
pub mod socket;

pub use event::HyprlandEvent;
pub use listener::EventListenerHandle;
