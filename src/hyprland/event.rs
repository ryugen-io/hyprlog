//! Hyprland event parsing.

use hypr_sdk::ipc::Event as SdkEvent;

/// A parsed Hyprland IPC event from socket2.
///
/// Events arrive as `EVENT_NAME>>DATA\n` on the event socket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HyprlandEvent {
    /// Event name (e.g., "openwindow", "workspace", "activewindow").
    pub name: String,
    /// Event data payload (may be empty, may contain multiple `>>` characters).
    pub data: String,
}

impl HyprlandEvent {
    /// Parses a raw event line from socket2.
    ///
    /// Format: `EVENT_NAME>>DATA` where DATA may be empty or contain `>>`.
    /// Returns `None` if the line doesn't contain `>>`.
    #[must_use]
    pub fn parse(line: &str) -> Option<Self> {
        let trimmed = line.trim();
        let (raw_name, raw_data) = trimmed.split_once(">>")?;
        let name = raw_name.trim();

        if name.is_empty() {
            return None;
        }

        Some(Self {
            name: name.to_string(),
            data: raw_data.to_string(),
        })
    }

    /// Converts a typed `hypr-sdk` event into a `HyprlandEvent`.
    #[must_use]
    pub fn from_sdk(event: &SdkEvent) -> Self {
        Self {
            name: event.wire_name().to_string(),
            data: event.wire_data(),
        }
    }

    /// Formats the event as a human-readable log message.
    ///
    /// Returns `"eventname: data"` or just `"eventname"` when data is empty.
    #[must_use]
    pub fn format_message(&self) -> String {
        if self.data.is_empty() {
            self.name.clone()
        } else {
            format!("{}: {}", self.name, self.data)
        }
    }
}
