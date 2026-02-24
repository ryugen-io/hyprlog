//! Raw IPC event names like "openwindow" and comma-delimited data are cryptic in logs —
//! translating them to "window opened" with key=value pairs makes log output scannable.

use super::event::HyprlandEvent;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Wire names like "openwindow" are terse IPC identifiers, not user-facing text —
/// this map provides the readable labels that appear in formatted log output.
static NAME_MAP: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::with_capacity(43);

    // Window lifecycle — user-visible actions that change what's on screen
    m.insert("openwindow", "window opened");
    m.insert("closewindow", "window closed");
    m.insert("movewindow", "window moved");
    m.insert("movewindowv2", "window moved");
    m.insert("windowtitle", "title changed");
    m.insert("windowtitlev2", "title changed");
    m.insert("activewindow", "window focused");
    m.insert("activewindowv2", "window focused");
    m.insert("urgent", "focus requested");

    // Window state — property toggles, not lifecycle transitions
    m.insert("fullscreen", "fullscreen toggled");
    m.insert("changefloatingmode", "float toggled");
    m.insert("minimized", "minimized");
    m.insert("pin", "pin toggled");

    // Workspace lifecycle — parallel to window events but at the workspace level
    m.insert("workspace", "workspace changed");
    m.insert("workspacev2", "workspace changed");
    m.insert("createworkspace", "workspace created");
    m.insert("createworkspacev2", "workspace created");
    m.insert("destroyworkspace", "workspace destroyed");
    m.insert("destroyworkspacev2", "workspace destroyed");
    m.insert("moveworkspace", "workspace moved");
    m.insert("moveworkspacev2", "workspace moved");
    m.insert("renameworkspace", "workspace renamed");

    // Special workspace — scratchpads and hidden workspaces behave differently from named ones
    m.insert("activespecial", "special workspace");
    m.insert("activespecialv2", "special workspace");

    // Monitor — display hotplug and focus changes affect multi-monitor workflows
    m.insert("focusedmon", "monitor focus");
    m.insert("focusedmonv2", "monitor focus");
    m.insert("monitoradded", "monitor added");
    m.insert("monitoraddedv2", "monitor added");
    m.insert("monitorremoved", "monitor removed");
    m.insert("monitorremovedv2", "monitor removed");

    // Groups — tabbed/grouped window layouts need their own event category
    m.insert("togglegroup", "group toggled");
    m.insert("lockgroups", "groups locked");
    m.insert("moveintogroup", "moved into group");
    m.insert("moveoutofgroup", "moved out of group");
    m.insert("ignoregrouplock", "group lock ignore");

    // Layers — overlay surfaces (bars, notifications) that sit above tiled windows
    m.insert("openlayer", "layer opened");
    m.insert("closelayer", "layer closed");

    // Input — keyboard layout and keymap state changes
    m.insert("activelayout", "layout changed");
    m.insert("submap", "submap");

    // Misc — events that don't fit the categories above
    m.insert("screencast", "screencast");
    m.insert("configreloaded", "config reloaded");
    m.insert("bell", "bell");

    m
});

/// Stateless formatting loses context.
///
/// A bare hex address in a closewindow event is meaningless without knowing
/// which app it belonged to. The window cache bridges that gap by remembering
/// openwindow-to-address associations.
pub struct EventFormatter {
    window_cache: HashMap<String, String>,
    human_readable: bool,
}

impl EventFormatter {
    /// Cache starts empty — it fills up as openwindow events arrive during the session.
    #[must_use]
    pub fn new(human_readable: bool) -> Self {
        Self {
            window_cache: HashMap::new(),
            human_readable,
        }
    }

    /// Must be called after `format()` — if called before, closewindow would evict the
    /// address before format has a chance to resolve it. The listener enforces this ordering.
    pub fn observe(&mut self, event: &HyprlandEvent) {
        match event.name.as_str() {
            "openwindow" => {
                // Wire format: addr,ws,class,title — we cache addr→class for later lookups
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

    /// Delegates to raw wire format when human-readable mode is off — lets users choose
    /// between debuggable output and polished log lines.
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
            // Each event has a unique wire format — dedicated formatters parse it correctly
            "openwindow" => self.format_openwindow(&event.data),
            "closewindow" | "urgent" | "windowtitle" | "activewindowv2" | "moveintogroup"
            | "moveoutofgroup" | "bell" => self.format_address_only(&event.data),
            "windowtitlev2" => Self::format_addr_comma_value(&event.data, "title"),
            "activewindow" => Self::format_activewindow(&event.data),
            "movewindow" => Self::format_addr_comma_value(&event.data, "ws"),
            "movewindowv2" => Self::format_movewindowv2(&event.data),

            // Boolean toggle events share the same 0/1 wire format
            "fullscreen" | "lockgroups" | "ignoregrouplock" => {
                Self::format_bool(&event.data, "enabled")
            }
            "changefloatingmode" => self.format_addr_bool(&event.data, "tiled"),
            "minimized" => self.format_addr_bool(&event.data, "minimized"),
            "pin" => self.format_addr_bool(&event.data, "pinned"),

            // v1 events carry only a name — no numeric ID available
            "workspace" | "createworkspace" | "destroyworkspace" | "monitoradded"
            | "monitorremoved" => {
                format!("name={}", event.data)
            }
            // v2 events carry both id and name — richer than v1
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

            // Monitor events vary between v1 (name-only) and v2 (id+name+description)
            "focusedmon" => Self::format_csv_kv(&event.data, &["monitor", "workspace"]),
            "focusedmonv2" => Self::format_csv_kv(&event.data, &["monitor", "id"]),
            "monitoraddedv2" | "monitorremovedv2" => {
                Self::format_csv_kv(&event.data, &["id", "name", "description"])
            }

            // Group toggle includes member addresses — count is more useful than raw hex
            "togglegroup" => Self::format_togglegroup(&event.data),

            // Layer events carry only a namespace string — no further parsing needed
            "openlayer" | "closelayer" => format!("namespace={}", event.data),

            // Keyboard layout switches are comma-separated key,value pairs
            "activelayout" => Self::format_csv_kv(&event.data, &["keyboard", "layout"]),

            // Screencast has a unique active+owner format unlike other boolean events
            "screencast" => Self::format_screencast(&event.data),

            _ => event.data.clone(),
        }
    }

    // --- Per-event parsers — each wire format needs its own split logic ---

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

    /// Titles may contain commas and spaces — quoting prevents ambiguity in key=value output.
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

    /// Address-prefixed boolean events need both cache lookup and 0/1→true/false translation.
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

    /// Hyprland sends 0/1 integers but log readers expect true/false booleans.
    fn format_bool(data: &str, key: &str) -> String {
        let val = if data.trim() == "1" { "true" } else { "false" };
        format!("{key}={val}")
    }

    /// v2 workspace events always carry id,name — consistent key=value output helps log parsing.
    fn format_id_name(data: &str) -> String {
        match data.split_once(',') {
            Some((id, name)) => format!("id={id} name={name}"),
            None => format!("name={data}"),
        }
    }

    /// Many events share the same comma-separated positional format — this avoids duplicating split logic.
    fn format_csv_kv(data: &str, keys: &[&str]) -> String {
        let fields: Vec<&str> = data.splitn(keys.len(), ',').collect();
        keys.iter()
            .zip(fields.iter())
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Raw group data dumps a variable-length address list — window count is more useful than hex addresses.
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

    /// Screencast events combine a boolean state with the owning application — both matter for debugging screen sharing issues.
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
