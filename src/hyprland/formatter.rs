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
    pub fn observe(&mut self, event: &HyprlandEvent) {
        match event.name.as_str() {
            "openwindow" => {
                // Format: addr,ws,class,title
                let mut fields = event.data.splitn(4, ',');
                if let (Some(addr), Some(_ws), Some(class)) =
                    (fields.next(), fields.next(), fields.next())
                {
                    let addr = addr.trim();
                    let class = class.trim();
                    if !addr.is_empty() && !class.is_empty() {
                        self.window_cache
                            .insert(addr.to_string(), class.to_ascii_lowercase());
                    }
                }
            }
            "closewindow" => {
                let addr = event.data.trim();
                if !addr.is_empty() {
                    self.window_cache.remove(addr);
                }
            }
            _ => {}
        }
    }

    /// Formats an event as a human-readable log message.
    #[must_use]
    pub fn format(&self, event: &HyprlandEvent) -> String {
        let human = NAME_MAP.get(event.name.as_str());

        if event.data.is_empty() {
            return human.map_or_else(|| event.name.clone(), |label| (*label).to_string());
        }

        let formatted_data = self.format_data(event);
        human.map_or_else(
            || format!("{}: {formatted_data}", event.name),
            |label| format!("{label} ({}): {formatted_data}", event.name),
        )
    }

    fn format_data(&self, event: &HyprlandEvent) -> String {
        match event.name.as_str() {
            "openwindow" => self.format_openwindow(&event.data),
            "closewindow" | "urgent" | "windowtitle" => self.format_address_only(&event.data),
            "windowtitlev2" => Self::format_windowtitlev2(&event.data),
            "focusedmonv2" => Self::format_focusedmonv2(&event.data),
            "movewindowv2" => Self::format_movewindowv2(&event.data),
            "movewindow" => Self::format_movewindow(&event.data),
            "activewindow" => Self::format_activewindow(&event.data),
            "workspace" | "createworkspace" | "destroyworkspace" => {
                format!("name={}", event.data)
            }
            _ => event.data.clone(),
        }
    }

    fn format_openwindow(&self, data: &str) -> String {
        let mut fields = data.splitn(4, ',');
        let addr = fields.next().unwrap_or("");
        let ws = fields.next().unwrap_or("");
        let class = fields.next().unwrap_or("");
        let title = fields.next().unwrap_or("");
        let app = self
            .window_cache
            .get(addr.trim())
            .map_or_else(|| class.trim().to_ascii_lowercase(), String::clone);
        format!(r#"app={app} title="{title}" ws={ws}"#)
    }

    fn format_address_only(&self, data: &str) -> String {
        let addr = data.trim();
        self.window_cache
            .get(addr)
            .map_or_else(|| addr.to_string(), |app| format!("app={app}"))
    }

    fn format_windowtitlev2(data: &str) -> String {
        match data.split_once(',') {
            Some((_addr, title)) => format!(r#"title="{title}""#),
            None => data.to_string(),
        }
    }

    fn format_focusedmonv2(data: &str) -> String {
        match data.split_once(',') {
            Some((name, id)) => format!("monitor={name} id={id}"),
            None => data.to_string(),
        }
    }

    fn format_movewindowv2(data: &str) -> String {
        let mut fields = data.splitn(3, ',');
        let _addr = fields.next();
        let _ws_id = fields.next();
        fields
            .next()
            .map_or_else(|| data.to_string(), |ws_name| format!("ws={ws_name}"))
    }

    fn format_movewindow(data: &str) -> String {
        match data.split_once(',') {
            Some((_addr, ws)) => format!("ws={ws}"),
            None => data.to_string(),
        }
    }

    fn format_activewindow(data: &str) -> String {
        match data.split_once(',') {
            Some((class, title)) => format!(r#"app={class} title="{title}""#),
            None => data.to_string(),
        }
    }
}

impl Default for EventFormatter {
    fn default() -> Self {
        Self::new()
    }
}
