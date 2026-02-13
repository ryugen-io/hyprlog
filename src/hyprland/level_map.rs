//! Event-to-log-level mapping for Hyprland events.

use crate::level::Level;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Cached default level map (built once).
static DEFAULT_LEVELS: LazyLock<HashMap<&'static str, Level>> = LazyLock::new(default_level_map);

/// Returns the default event name to log level mapping.
///
/// Categorizes Hyprland events by their significance:
/// - **Info**: Window/workspace lifecycle events (user-visible actions)
/// - **Debug**: Focus changes, layout updates, config reloads (high-frequency)
/// - **Warn**: Events requiring attention (urgent flag)
#[must_use]
pub fn default_level_map() -> HashMap<&'static str, Level> {
    let mut map = HashMap::new();

    // Info: window/workspace lifecycle
    for name in [
        "openwindow",
        "closewindow",
        "movewindow",
        "workspace",
        "createworkspace",
        "destroyworkspace",
        "moveworkspace",
        "renameworkspace",
        "monitoradded",
        "monitorremoved",
        "submap",
    ] {
        map.insert(name, Level::Info);
    }

    // Debug: focus/layout changes, high-frequency
    for name in [
        "activewindow",
        "activewindowv2",
        "focusedmon",
        "activelayout",
        "fullscreen",
        "changefloatingmode",
        "configreloaded",
        "openlayer",
        "closelayer",
        "screencast",
        "pin",
    ] {
        map.insert(name, Level::Debug);
    }

    // Warn: attention-requiring
    map.insert("urgent", Level::Warn);

    map
}

/// Resolves the log level for an event.
///
/// Priority: user overrides (from config) > default map > fallback (Info).
#[must_use]
pub fn resolve_level<S: ::std::hash::BuildHasher>(
    event_name: &str,
    user_overrides: &HashMap<String, String, S>,
) -> Level {
    // Check user overrides first
    if let Some(level_str) = user_overrides.get(event_name) {
        if let Ok(level) = level_str.parse() {
            return level;
        }
    }

    // Check default map (cached)
    DEFAULT_LEVELS
        .get(event_name)
        .copied()
        .unwrap_or(Level::Info)
}
