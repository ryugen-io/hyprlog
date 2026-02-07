//! Hyprland IPC integration.
//!
//! Provides direct access to Hyprland's Unix domain sockets:
//! - **Socket1** (command socket): Send hyprctl-style queries and dispatcher commands
//! - **Socket2** (event socket): Listen to live compositor events
//!
//! # Example
//!
//! ```no_run
//! use hyprlog::hyprland::{command, listener};
//! use hyprlog::config::HyprlandConfig;
//!
//! let config = HyprlandConfig::default();
//!
//! // Query Hyprland
//! let monitors = command::query(&config, "monitors").unwrap();
//! println!("{monitors}");
//!
//! // Dispatch a command
//! command::dispatch(&config, "workspace 2").unwrap();
//! ```

pub mod command;
pub mod error;
pub mod event;
pub mod level_map;
pub mod listener;
pub mod socket;

pub use error::HyprlandError;
pub use event::HyprlandEvent;
pub use listener::EventListenerHandle;
