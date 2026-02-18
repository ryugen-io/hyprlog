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

    /// Converts a typed `hypr-sdk` event into a `HyprlandEvent`.
    #[must_use]
    pub fn from_sdk(event: &SdkEvent) -> Self {
        Self {
            name: canonical_event_name(event).into_owned(),
            data: event_data(event),
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

fn event_data(event: &SdkEvent) -> String {
    match event {
        SdkEvent::Workspace { name }
        | SdkEvent::CreateWorkspace { name }
        | SdkEvent::DestroyWorkspace { name }
        | SdkEvent::MonitorAdded { name }
        | SdkEvent::MonitorRemoved { name }
        | SdkEvent::OpenLayer { namespace: name }
        | SdkEvent::CloseLayer { namespace: name }
        | SdkEvent::Submap { name } => name.clone(),
        SdkEvent::WorkspaceV2 { id, name }
        | SdkEvent::CreateWorkspaceV2 { id, name }
        | SdkEvent::DestroyWorkspaceV2 { id, name } => format!("{id},{name}"),
        SdkEvent::MoveWorkspace { name, monitor } => format!("{name},{monitor}"),
        SdkEvent::MoveWorkspaceV2 { id, name, monitor } => format!("{id},{name},{monitor}"),
        SdkEvent::RenameWorkspace { id, new_name } => format!("{id},{new_name}"),
        SdkEvent::FocusedMon { monitor, workspace } => format!("{monitor},{workspace}"),
        SdkEvent::FocusedMonV2 {
            monitor,
            workspace_id,
        } => format!("{monitor},{workspace_id}"),
        SdkEvent::MonitorAddedV2 {
            id,
            name,
            description,
        }
        | SdkEvent::MonitorRemovedV2 {
            id,
            name,
            description,
        } => format!("{id},{name},{description}"),
        SdkEvent::ActiveSpecial { name, monitor } => format!("{name},{monitor}"),
        SdkEvent::ActiveSpecialV2 { id, name, monitor } => format!("{id},{name},{monitor}"),
        SdkEvent::ActiveWindow { class, title } => format!("{class},{title}"),
        SdkEvent::ActiveWindowV2 { address }
        | SdkEvent::CloseWindow { address }
        | SdkEvent::WindowTitle { address }
        | SdkEvent::Urgent { address }
        | SdkEvent::MoveIntoGroup { address }
        | SdkEvent::MoveOutOfGroup { address } => format_address(address),
        SdkEvent::OpenWindow {
            address,
            workspace,
            class,
            title,
        } => format!("{},{workspace},{class},{title}", format_address(address)),
        SdkEvent::WindowTitleV2 { address, title } => {
            format!("{},{}", format_address(address), title)
        }
        SdkEvent::MoveWindow { address, workspace } => {
            format!("{},{}", format_address(address), workspace)
        }
        SdkEvent::MoveWindowV2 {
            address,
            workspace_id,
            workspace_name,
        } => format!(
            "{},{workspace_id},{workspace_name}",
            format_address(address)
        ),
        SdkEvent::Fullscreen { enabled } => bool_as_int(*enabled).to_string(),
        SdkEvent::ChangeFloatingMode { address, is_tiled } => {
            format!("{},{}", format_address(address), bool_as_int(*is_tiled))
        }
        SdkEvent::Minimized { address, minimized } => {
            format!("{},{}", format_address(address), bool_as_int(*minimized))
        }
        SdkEvent::Pin { address, pinned } => {
            format!("{},{}", format_address(address), bool_as_int(*pinned))
        }
        SdkEvent::ToggleGroup { state, addresses } => {
            let mut data = bool_as_int(*state).to_string();
            if !addresses.is_empty() {
                data.push(',');
                data.push_str(
                    &addresses
                        .iter()
                        .map(format_address)
                        .collect::<Vec<_>>()
                        .join(","),
                );
            }
            data
        }
        SdkEvent::LockGroups { locked } => bool_as_int(*locked).to_string(),
        SdkEvent::IgnoreGroupLock { enabled } => bool_as_int(*enabled).to_string(),
        SdkEvent::ActiveLayout { keyboard, layout } => format!("{keyboard},{layout}"),
        SdkEvent::Bell { address } => address.clone(),
        SdkEvent::Screencast { active, owner } => format!("{},{}", bool_as_int(*active), owner),
        SdkEvent::ConfigReloaded => String::new(),
        SdkEvent::Custom { data } => data.clone(),
        SdkEvent::Unknown { data, .. } => data.clone(),
    }
}

const fn bool_as_int(value: bool) -> u8 {
    if value { 1 } else { 0 }
}

fn format_address(address: &hypr_sdk::types::common::WindowAddress) -> String {
    format!("{:x}", address.0)
}
