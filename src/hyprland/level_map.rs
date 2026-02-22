//! Not all Hyprland events deserve the same severity — window lifecycle is Info,
//! focus changes are Debug, and `urgent` is Warn. Users can override any mapping.

use crate::level::Level;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Built once at first access — avoids reconstructing the map on every event.
static DEFAULT_LEVELS: LazyLock<HashMap<&'static str, Level>> = LazyLock::new(default_level_map);

/// Sensible defaults so events log at appropriate severity without per-event config.
/// Info for user-visible actions, Debug for high-frequency noise, Warn for urgent.
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

/// User overrides take priority over defaults — unknown events fall back to Info
/// since most custom events represent normal operational activity.
#[must_use]
pub fn resolve_level<S: ::std::hash::BuildHasher>(
    event_name: &str,
    user_overrides: &HashMap<String, String, S>,
) -> Level {
    // Check user overrides first
    if let Some(level_str) = user_overrides.get(event_name)
        && let Ok(level) = level_str.parse()
    {
        return level;
    }

    // Check default map (cached)
    DEFAULT_LEVELS
        .get(event_name)
        .copied()
        .unwrap_or(Level::Info)
}
