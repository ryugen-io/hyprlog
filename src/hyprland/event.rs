//! Hyprland event parsing.

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
        let idx = trimmed.find(">>")?;
        let name = trimmed[..idx].to_string();
        let data = trimmed[idx + 2..].to_string();
        if name.is_empty() {
            return None;
        }
        Some(Self { name, data })
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
