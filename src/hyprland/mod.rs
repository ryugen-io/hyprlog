//! Hyprland exposes compositor events over Unix domain sockets — this module
//! connects to socket2, parses the wire format, and routes events through
//! the logger so they appear alongside application logs.

pub mod event;
pub mod formatter;
pub mod level_map;
pub mod listener;
pub mod socket;

pub use event::HyprlandEvent;
pub use formatter::EventFormatter;
pub use listener::EventListenerHandle;
