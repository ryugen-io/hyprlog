//! Human-readable formatting for Hyprland events.

use super::event::HyprlandEvent;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Human-readable labels for known Hyprland event names.
static NAME_MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::with_capacity(43);

    // Window lifecycle
    m.insert("openwindow", "window opened");
    m.insert("closewindow", "window closed");
    m.insert("movewindow", "window moved");
    m.insert("movewindowv2", "window moved");
    m.insert("windowtitle", "title changed");
    m.insert("windowtitlev2", "title changed");
    m.insert("activewindow", "window focused");
    m.insert("activewindowv2", "window focused");
    m.insert("urgent", "focus requested");

    // Window state
    m.insert("fullscreen", "fullscreen toggled");
    m.insert("changefloatingmode", "float toggled");
    m.insert("minimized", "minimized");
    m.insert("pin", "pin toggled");

    // Workspace lifecycle
    m.insert("workspace", "workspace changed");
    m.insert("workspacev2", "workspace changed");
    m.insert("createworkspace", "workspace created");
    m.insert("createworkspacev2", "workspace created");
    m.insert("destroyworkspace", "workspace destroyed");
    m.insert("destroyworkspacev2", "workspace destroyed");
    m.insert("moveworkspace", "workspace moved");
    m.insert("moveworkspacev2", "workspace moved");
    m.insert("renameworkspace", "workspace renamed");

    // Special workspace
    m.insert("activespecial", "special workspace");
    m.insert("activespecialv2", "special workspace");

    // Monitor
    m.insert("focusedmon", "monitor focus");
    m.insert("focusedmonv2", "monitor focus");
    m.insert("monitoradded", "monitor added");
    m.insert("monitoraddedv2", "monitor added");
    m.insert("monitorremoved", "monitor removed");
    m.insert("monitorremovedv2", "monitor removed");

    // Groups
    m.insert("togglegroup", "group toggled");
    m.insert("lockgroups", "groups locked");
    m.insert("moveintogroup", "moved into group");
    m.insert("moveoutofgroup", "moved out of group");
    m.insert("ignoregrouplock", "group lock ignore");

    // Layers
    m.insert("openlayer", "layer opened");
    m.insert("closelayer", "layer closed");

    // Input
    m.insert("activelayout", "layout changed");
    m.insert("submap", "submap");

    // Misc
    m.insert("screencast", "screencast");
    m.insert("configreloaded", "config reloaded");
    m.insert("bell", "bell");

    m
});

/// Formats Hyprland events with human-readable names and parsed data.
///
/// Maintains a window-address cache to resolve hex addresses to app names.
pub struct EventFormatter {
    window_cache: HashMap<String, String>,
    human_readable: bool,
}

impl EventFormatter {
    /// Creates a new formatter with an empty window cache.
    #[must_use]
    pub fn new(human_readable: bool) -> Self {
        Self {
            window_cache: HashMap::new(),
            human_readable,
        }
    }

    /// Updates internal caches based on the event.
    ///
    /// Call this after `format()` for every event to keep the window cache current.
    /// The listener calls format-then-observe so closewindow can still resolve cached app names.
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
        if !self.human_readable {
            return event.format_message();
        }

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
            // Window lifecycle
            "openwindow" => self.format_openwindow(&event.data),
            "closewindow" | "urgent" | "windowtitle" | "activewindowv2" | "moveintogroup"
            | "moveoutofgroup" | "bell" => self.format_address_only(&event.data),
            "windowtitlev2" => Self::format_addr_comma_value(&event.data, "title"),
            "activewindow" => Self::format_activewindow(&event.data),
            "movewindow" => Self::format_addr_comma_value(&event.data, "ws"),
            "movewindowv2" => Self::format_movewindowv2(&event.data),

            // Window state
            "fullscreen" | "lockgroups" | "ignoregrouplock" => {
                Self::format_bool(&event.data, "enabled")
            }
            "changefloatingmode" => self.format_addr_bool(&event.data, "tiled"),
            "minimized" => self.format_addr_bool(&event.data, "minimized"),
            "pin" => self.format_addr_bool(&event.data, "pinned"),

            // Workspace (name only) / monitor (name only)
            "workspace" | "createworkspace" | "destroyworkspace" | "monitoradded"
            | "monitorremoved" => {
                format!("name={}", event.data)
            }
            // Workspace v2 (id,name)
            "workspacev2" | "createworkspacev2" | "destroyworkspacev2" => {
                Self::format_id_name(&event.data)
            }
            "moveworkspace" | "activespecial" => {
                Self::format_csv_kv(&event.data, &["name", "monitor"])
            }
            "moveworkspacev2" | "activespecialv2" => {
                Self::format_csv_kv(&event.data, &["id", "name", "monitor"])
            }
            "renameworkspace" => Self::format_csv_kv(&event.data, &["id", "name"]),

            // Monitor
            "focusedmon" => Self::format_csv_kv(&event.data, &["monitor", "workspace"]),
            "focusedmonv2" => Self::format_csv_kv(&event.data, &["monitor", "id"]),
            "monitoraddedv2" | "monitorremovedv2" => {
                Self::format_csv_kv(&event.data, &["id", "name", "description"])
            }

            // Groups
            "togglegroup" => Self::format_togglegroup(&event.data),

            // Layers
            "openlayer" | "closelayer" => format!("namespace={}", event.data),

            // Input
            "activelayout" => Self::format_csv_kv(&event.data, &["keyboard", "layout"]),

            // Misc
            "screencast" => Self::format_screencast(&event.data),

            _ => event.data.clone(),
        }
    }

    // --- Formatters ---

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

    /// Formats `addr,value` as `{key}="{value}"` (quoted) or `{key}={value}` for ws.
    fn format_addr_comma_value(data: &str, key: &str) -> String {
        match data.split_once(',') {
            Some((_addr, value)) => {
                if key == "title" {
                    format!(r#"{key}="{value}""#)
                } else {
                    format!("{key}={value}")
                }
            }
            None => data.to_string(),
        }
    }

    /// Formats `addr,0|1` as `app={cached} {key}={true|false}`.
    fn format_addr_bool(&self, data: &str, key: &str) -> String {
        match data.split_once(',') {
            Some((addr, flag)) => {
                let bool_val = if flag.trim() == "1" { "true" } else { "false" };
                let app_part = self
                    .window_cache
                    .get(addr.trim())
                    .map_or(String::new(), |app| format!("app={app} "));
                format!("{app_part}{key}={bool_val}")
            }
            None => data.to_string(),
        }
    }

    fn format_activewindow(data: &str) -> String {
        match data.split_once(',') {
            Some((class, title)) => format!(r#"app={class} title="{title}""#),
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

    /// Formats a single `0|1` as `{key}={true|false}`.
    fn format_bool(data: &str, key: &str) -> String {
        let val = if data.trim() == "1" { "true" } else { "false" };
        format!("{key}={val}")
    }

    /// Formats `id,name` as `id={id} name={name}`.
    fn format_id_name(data: &str) -> String {
        match data.split_once(',') {
            Some((id, name)) => format!("id={id} name={name}"),
            None => format!("name={data}"),
        }
    }

    /// Generic CSV to key=value formatter.
    fn format_csv_kv(data: &str, keys: &[&str]) -> String {
        let fields: Vec<&str> = data.splitn(keys.len(), ',').collect();
        keys.iter()
            .zip(fields.iter())
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Formats `0|1,addr1,addr2,...` as `state={opened|closed} windows=N`.
    fn format_togglegroup(data: &str) -> String {
        if let Some((flag, rest)) = data.split_once(',') {
            let state = if flag.trim() == "1" {
                "opened"
            } else {
                "closed"
            };
            let count = rest.split(',').count();
            format!("state={state} windows={count}")
        } else {
            let state = if data.trim() == "1" {
                "opened"
            } else {
                "closed"
            };
            format!("state={state}")
        }
    }

    /// Formats `0|1,owner` as `active={true|false} owner={owner}`.
    fn format_screencast(data: &str) -> String {
        match data.split_once(',') {
            Some((flag, owner)) => {
                let active = if flag.trim() == "1" { "true" } else { "false" };
                format!("active={active} owner={owner}")
            }
            None => data.to_string(),
        }
    }
}

impl Default for EventFormatter {
    fn default() -> Self {
        Self::new(true)
    }
}
