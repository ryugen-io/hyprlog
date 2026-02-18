//! Hyprland event parsing.

use hypr_sdk::ipc::Event as SdkEvent;
use hypr_sdk::ipc::events;
use std::borrow::Cow;

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

        if raw_name.is_empty() {
            return None;
        }

        let name = events::parse_event(trimmed)
            .map(|event| canonical_event_name(&event).into_owned())
            .unwrap_or_else(|| raw_name.to_string());

        Some(Self {
            name,
            data: raw_data.to_string(),
        })
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

fn canonical_event_name(event: &SdkEvent) -> Cow<'_, str> {
    match event {
        SdkEvent::Workspace { .. } => Cow::Borrowed("workspace"),
        SdkEvent::WorkspaceV2 { .. } => Cow::Borrowed("workspacev2"),
        SdkEvent::CreateWorkspace { .. } => Cow::Borrowed("createworkspace"),
        SdkEvent::CreateWorkspaceV2 { .. } => Cow::Borrowed("createworkspacev2"),
        SdkEvent::DestroyWorkspace { .. } => Cow::Borrowed("destroyworkspace"),
        SdkEvent::DestroyWorkspaceV2 { .. } => Cow::Borrowed("destroyworkspacev2"),
        SdkEvent::MoveWorkspace { .. } => Cow::Borrowed("moveworkspace"),
        SdkEvent::MoveWorkspaceV2 { .. } => Cow::Borrowed("moveworkspacev2"),
        SdkEvent::RenameWorkspace { .. } => Cow::Borrowed("renameworkspace"),
        SdkEvent::FocusedMon { .. } => Cow::Borrowed("focusedmon"),
        SdkEvent::FocusedMonV2 { .. } => Cow::Borrowed("focusedmonv2"),
        SdkEvent::MonitorAdded { .. } => Cow::Borrowed("monitoradded"),
        SdkEvent::MonitorAddedV2 { .. } => Cow::Borrowed("monitoraddedv2"),
        SdkEvent::MonitorRemoved { .. } => Cow::Borrowed("monitorremoved"),
        SdkEvent::MonitorRemovedV2 { .. } => Cow::Borrowed("monitorremovedv2"),
        SdkEvent::ActiveSpecial { .. } => Cow::Borrowed("activespecial"),
        SdkEvent::ActiveSpecialV2 { .. } => Cow::Borrowed("activespecialv2"),
        SdkEvent::ActiveWindow { .. } => Cow::Borrowed("activewindow"),
        SdkEvent::ActiveWindowV2 { .. } => Cow::Borrowed("activewindowv2"),
        SdkEvent::OpenWindow { .. } => Cow::Borrowed("openwindow"),
        SdkEvent::CloseWindow { .. } => Cow::Borrowed("closewindow"),
        SdkEvent::WindowTitle { .. } => Cow::Borrowed("windowtitle"),
        SdkEvent::WindowTitleV2 { .. } => Cow::Borrowed("windowtitlev2"),
        SdkEvent::MoveWindow { .. } => Cow::Borrowed("movewindow"),
        SdkEvent::MoveWindowV2 { .. } => Cow::Borrowed("movewindowv2"),
        SdkEvent::Fullscreen { .. } => Cow::Borrowed("fullscreen"),
        SdkEvent::ChangeFloatingMode { .. } => Cow::Borrowed("changefloatingmode"),
        SdkEvent::Urgent { .. } => Cow::Borrowed("urgent"),
        SdkEvent::Minimized { .. } => Cow::Borrowed("minimized"),
        SdkEvent::Pin { .. } => Cow::Borrowed("pin"),
        SdkEvent::ToggleGroup { .. } => Cow::Borrowed("togglegroup"),
        SdkEvent::LockGroups { .. } => Cow::Borrowed("lockgroups"),
        SdkEvent::MoveIntoGroup { .. } => Cow::Borrowed("moveintogroup"),
        SdkEvent::MoveOutOfGroup { .. } => Cow::Borrowed("moveoutofgroup"),
        SdkEvent::IgnoreGroupLock { .. } => Cow::Borrowed("ignoregrouplock"),
        SdkEvent::OpenLayer { .. } => Cow::Borrowed("openlayer"),
        SdkEvent::CloseLayer { .. } => Cow::Borrowed("closelayer"),
        SdkEvent::ActiveLayout { .. } => Cow::Borrowed("activelayout"),
        SdkEvent::Submap { .. } => Cow::Borrowed("submap"),
        SdkEvent::Bell { .. } => Cow::Borrowed("bell"),
        SdkEvent::Screencast { .. } => Cow::Borrowed("screencast"),
        SdkEvent::ConfigReloaded => Cow::Borrowed("configreloaded"),
        SdkEvent::Custom { .. } => Cow::Borrowed("custom"),
        SdkEvent::Unknown { name, .. } => Cow::Borrowed(name.as_str()),
    }
}
