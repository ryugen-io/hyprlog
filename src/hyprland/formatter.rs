//! Human-readable formatting for Hyprland events.

use super::event::HyprlandEvent;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Human-readable labels for known Hyprland event names.
static NAME_MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("openwindow", "window opened");
    m.insert("closewindow", "window closed");
    m.insert("movewindow", "window moved");
    m.insert("movewindowv2", "window moved");
    m.insert("windowtitle", "title changed");
    m.insert("windowtitlev2", "title changed");
    m.insert("focusedmon", "monitor focus");
    m.insert("focusedmonv2", "monitor focus");
    m.insert("workspace", "workspace changed");
    m.insert("createworkspace", "workspace created");
    m.insert("destroyworkspace", "workspace destroyed");
    m.insert("moveworkspace", "workspace moved");
    m.insert("renameworkspace", "workspace renamed");
    m.insert("activewindow", "window focused");
    m.insert("activewindowv2", "window focused");
    m.insert("urgent", "focus requested");
    m.insert("fullscreen", "fullscreen");
    m.insert("submap", "submap");
    m.insert("monitoradded", "monitor added");
    m.insert("monitorremoved", "monitor removed");
    m
});

/// Formats Hyprland events with human-readable names and parsed data.
///
/// Maintains a window-address cache to resolve hex addresses to app names.
pub struct EventFormatter {
    // Used in Task 2 for window address resolution.
    #[allow(dead_code)]
    window_cache: HashMap<String, String>,
}

impl EventFormatter {
    /// Creates a new formatter with an empty window cache.
    #[must_use]
    pub fn new() -> Self {
        Self {
            window_cache: HashMap::new(),
        }
    }

    /// Updates internal caches based on the event.
    ///
    /// Call this before `format()` for every event to keep the window cache current.
    #[allow(clippy::missing_const_for_fn)]
    pub fn observe(&mut self, _event: &HyprlandEvent) {
        // Window cache logic added in Task 2.
    }

    /// Formats an event as a human-readable log message.
    #[must_use]
    pub fn format(&self, event: &HyprlandEvent) -> String {
        let human = NAME_MAP.get(event.name.as_str());

        match (human, event.data.is_empty()) {
            (Some(label), true) => (*label).to_string(),
            (Some(label), false) => format!("{label} ({}): {}", event.name, event.data),
            (None, true) => event.name.clone(),
            (None, false) => format!("{}: {}", event.name, event.data),
        }
    }
}

impl Default for EventFormatter {
    fn default() -> Self {
        Self::new()
    }
}
