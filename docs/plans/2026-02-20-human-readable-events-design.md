# Human-Readable Hyprland Events + Window Cache

## Problem

Hyprland events are logged with raw technical names (`windowtitlev2`, `focusedmonv2`) and unparsed CSV data containing hex window addresses. Events like `urgent` show only a hex address with no context about which app triggered them.

## Solution

New `src/hyprland/formatter.rs` module with `EventFormatter` struct that:
1. Translates technical event names to human-readable labels
2. Parses CSV event data into key-value pairs
3. Maintains a window-address cache to resolve hex IDs to app names

## Output Format

```
{human_name} ({technical_name}): {key=value pairs}
```

### Examples

```
window opened (openwindow): app=rio title="▲" ws=1
title changed (windowtitlev2): title="Yazi: ~/"
focus requested (urgent): app=brave-browser
monitor focus (focusedmonv2): monitor=DP-2 id=2
window closed (closewindow): app=rio
window moved (movewindowv2): ws=2
```

## Event Name Map

| Technical | Human-Readable |
|-----------|---------------|
| openwindow | window opened |
| closewindow | window closed |
| movewindow / movewindowv2 | window moved |
| windowtitle / windowtitlev2 | title changed |
| focusedmon / focusedmonv2 | monitor focus |
| workspace | workspace changed |
| createworkspace | workspace created |
| destroyworkspace | workspace destroyed |
| activewindow / activewindowv2 | window focused |
| urgent | focus requested |
| fullscreen | fullscreen |
| submap | submap |
| monitoradded | monitor added |
| monitorremoved | monitor removed |
| moveworkspace | workspace moved |
| renameworkspace | workspace renamed |

Unknown events fall back to their technical name.

## Key-Value Data Parsing

Per-event CSV field extraction:

- **openwindow** `addr,ws,class,title` -> `app={class} title="{title}" ws={ws}`
- **closewindow** `addr` -> `app={cached_name}` (from window cache)
- **windowtitlev2** `addr,title` -> `title="{title}"`
- **windowtitle** `addr` -> (empty or cached app)
- **focusedmonv2** `name,id` -> `monitor={name} id={id}`
- **movewindowv2** `addr,ws_id,ws_name` -> `ws={ws_name}`
- **movewindow** `addr,ws` -> `ws={ws}`
- **urgent** `addr` -> `app={cached_name}` (from window cache)
- **activewindow** `class,title` -> `app={class} title="{title}"`
- **workspace** / **createworkspace** / **destroyworkspace** `name` -> `name={name}`
- **Fallback** -> raw data unchanged

## Window Address Cache

`EventFormatter` holds `HashMap<String, String>` mapping window address -> app name.

- **Populate**: on `openwindow` events, extract address and class fields
- **Remove**: on `closewindow` events, remove the address entry
- **Lookup**: on `urgent`, `closewindow`, `windowtitle`, `windowtitlev2`, `movewindow`, `movewindowv2` - resolve address to app name

## Architecture

### New file: `src/hyprland/formatter.rs`

```rust
pub struct EventFormatter {
    window_cache: HashMap<String, String>,
}

impl EventFormatter {
    pub fn new() -> Self;
    pub fn observe(&mut self, event: &HyprlandEvent);  // update cache
    pub fn format(&self, event: &HyprlandEvent) -> String;  // human-readable output
}
```

### Changes to `src/hyprland/listener.rs`

In `process_events()`:
- Create `EventFormatter` at loop start
- Before logging: `formatter.observe(&event)` to update window cache
- Replace `event.format_message()` with `formatter.format(&event)`

### No changes to

- `src/hyprland/event.rs` - `format_message()` remains as raw fallback
- `src/hyprland/level_map.rs` - level mapping unchanged
- `src/config/structs.rs` - no new config fields (hardcoded default for now)

## Design Decisions

- **Hardcoded default**: No config toggle for now. Can be added later as `[hyprland] human_readable = true/false`.
- **Formatter module**: Keeps listener clean, cache has clear ownership, testable in isolation.
- **Both names shown**: `human (technical)` format keeps output grep-able by Hyprland docs terminology.
- **Cache cleanup on closewindow**: Prevents unbounded growth during long sessions.
