//! Raw IPC lines like `openwindow>>ADDR,WS,CLASS,TITLE` need parsing into typed
//! structs before the logger or formatter can work with them.

use hypr_sdk::ipc::Event as SdkEvent;

/// Events arrive as `EVENT_NAME>>DATA\n` on socket2 — this struct splits them
/// into name and payload so downstream code doesn't re-parse the raw line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HyprlandEvent {
    /// Used for filtering, level mapping, and scope routing — the primary identity of the event.
    pub name: String,
    /// Payload format varies per event — comma-separated fields, single values, or empty.
    pub data: String,
}

impl HyprlandEvent {
    /// Splits the raw `EVENT>>DATA` wire format — the `>>` delimiter may appear in the data itself,
    /// so only the first occurrence is used as the split point.
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

    /// The SDK provides typed events — converting to our format decouples hyprlog from SDK internals.
    #[must_use]
    pub fn from_sdk(event: &SdkEvent) -> Self {
        Self {
            name: event.wire_name().to_string(),
            data: event.wire_data(),
        }
    }

    /// Raw wire format is useful for debugging but unreadable for end users — this adds structure.
    #[must_use]
    pub fn format_message(&self) -> String {
        if self.data.is_empty() {
            self.name.clone()
        } else {
            format!("{}: {}", self.name, self.data)
        }
    }
}
