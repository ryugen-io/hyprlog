# Human-Readable Hyprland Events Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace raw Hyprland event output with human-readable names and key-value formatted data, using a window-address cache to resolve hex IDs to app names.

**Architecture:** New `src/hyprland/formatter.rs` module containing `EventFormatter` struct with a static name map, per-event data parsers, and a `HashMap<String, String>` window-address cache. Integrated into `listener.rs` via `observe()` + `format()` calls replacing `event.format_message()`.

**Tech Stack:** Rust std (`HashMap`, `LazyLock`), existing `HyprlandEvent` struct from `src/hyprland/event.rs`.

---

### Task 1: EventFormatter — Name Map + Basic Format

**Files:**
- Create: `src/hyprland/formatter.rs`
- Modify: `src/hyprland/mod.rs`
- Test: `tests/hyprland_formatter.rs`

**Step 1: Write the failing tests**

Create `tests/hyprland_formatter.rs`:

```rust
//! Tests for the Hyprland event formatter.

#![cfg(feature = "hyprland")]

use hyprlog::HyprlandEvent;
use hyprlog::hyprland::formatter::EventFormatter;

#[test]
fn format_known_event_shows_human_name_and_technical() {
    let fmt = EventFormatter::new();
    let event = HyprlandEvent::parse("workspace>>3").unwrap();
    assert_eq!(fmt.format(&event), "workspace changed (workspace): 3");
}

#[test]
fn format_unknown_event_uses_technical_name() {
    let fmt = EventFormatter::new();
    let event = HyprlandEvent::parse("somenewevent>>payload").unwrap();
    assert_eq!(fmt.format(&event), "somenewevent: payload");
}

#[test]
fn format_empty_data_omits_colon() {
    let fmt = EventFormatter::new();
    let event = HyprlandEvent::parse("configreloaded>>").unwrap();
    assert_eq!(fmt.format(&event), "configreloaded");
}

#[test]
fn format_known_event_empty_data_omits_colon() {
    let fmt = EventFormatter::new();
    let event = HyprlandEvent { name: "fullscreen".into(), data: String::new() };
    assert_eq!(fmt.format(&event), "fullscreen");
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --features hyprland --test hyprland_formatter -- --nocapture 2>&1 | head -30`
Expected: Compilation error — `formatter` module doesn't exist.

**Step 3: Write minimal implementation**

Create `src/hyprland/formatter.rs`:

```rust
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
```

Add module to `src/hyprland/mod.rs` — insert `pub mod formatter;` and `pub use formatter::EventFormatter;`.

**Step 4: Run tests to verify they pass**

Run: `cargo test --features hyprland --test hyprland_formatter -v`
Expected: All 4 tests PASS.

**Step 5: Run lints**

Run: `cargo clippy --features hyprland -- -D warnings`
Expected: Clean.

**Step 6: Commit**

```bash
git add src/hyprland/formatter.rs src/hyprland/mod.rs tests/hyprland_formatter.rs
git commit -m "feat(hyprland): add EventFormatter with human-readable name map"
```

---

### Task 2: Window Address Cache

**Files:**
- Modify: `src/hyprland/formatter.rs`
- Test: `tests/hyprland_formatter.rs`

**Step 1: Write the failing tests**

Append to `tests/hyprland_formatter.rs`:

```rust
#[test]
fn observe_openwindow_populates_cache() {
    let mut fmt = EventFormatter::new();
    let open = HyprlandEvent::parse("openwindow>>80a6f50,2,kitty,Kitty").unwrap();
    fmt.observe(&open);

    let urgent = HyprlandEvent::parse("urgent>>80a6f50").unwrap();
    let msg = fmt.format(&urgent);
    assert_eq!(msg, "focus requested (urgent): app=kitty");
}

#[test]
fn observe_closewindow_removes_cache_entry() {
    let mut fmt = EventFormatter::new();
    let open = HyprlandEvent::parse("openwindow>>80a6f50,2,kitty,Kitty").unwrap();
    fmt.observe(&open);

    let close = HyprlandEvent::parse("closewindow>>80a6f50").unwrap();
    fmt.observe(&close);

    let urgent = HyprlandEvent::parse("urgent>>80a6f50").unwrap();
    let msg = fmt.format(&urgent);
    // No cached app, falls back to raw data
    assert_eq!(msg, "focus requested (urgent): 80a6f50");
}

#[test]
fn urgent_without_cache_shows_raw_address() {
    let fmt = EventFormatter::new();
    let event = HyprlandEvent::parse("urgent>>80a6f50").unwrap();
    assert_eq!(fmt.format(&event), "focus requested (urgent): 80a6f50");
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --features hyprland --test hyprland_formatter -- observe 2>&1 | head -20`
Expected: `observe_openwindow_populates_cache` FAILS — `format()` doesn't use cache yet.

**Step 3: Implement window cache**

In `src/hyprland/formatter.rs`, update `observe()`:

```rust
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
```

Update `format()` to use cache for address-only events:

```rust
pub fn format(&self, event: &HyprlandEvent) -> String {
    let human = NAME_MAP.get(event.name.as_str());

    if event.data.is_empty() {
        return human.map_or_else(|| event.name.clone(), |label| (*label).to_string());
    }

    let formatted_data = self.format_data(event);
    match human {
        Some(label) => format!("{label} ({}): {formatted_data}", event.name),
        None => format!("{}: {formatted_data}", event.name),
    }
}

fn format_data(&self, event: &HyprlandEvent) -> String {
    match event.name.as_str() {
        "urgent" => self.format_address_only(&event.data),
        _ => event.data.clone(),
    }
}

fn format_address_only(&self, data: &str) -> String {
    let addr = data.trim();
    match self.window_cache.get(addr) {
        Some(app) => format!("app={app}"),
        None => addr.to_string(),
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --features hyprland --test hyprland_formatter -v`
Expected: All 7 tests PASS.

**Step 5: Run lints**

Run: `cargo clippy --features hyprland -- -D warnings`

**Step 6: Commit**

```bash
git add src/hyprland/formatter.rs tests/hyprland_formatter.rs
git commit -m "feat(hyprland): add window-address cache to EventFormatter"
```

---

### Task 3: Key-Value Data Parsing For All Events

**Files:**
- Modify: `src/hyprland/formatter.rs`
- Test: `tests/hyprland_formatter.rs`

**Step 1: Write the failing tests**

Append to `tests/hyprland_formatter.rs`:

```rust
#[test]
fn format_openwindow_key_value() {
    let mut fmt = EventFormatter::new();
    let event = HyprlandEvent::parse("openwindow>>80a6f50,2,kitty,Kitty Terminal").unwrap();
    fmt.observe(&event);
    assert_eq!(
        fmt.format(&event),
        r#"window opened (openwindow): app=kitty title="Kitty Terminal" ws=2"#
    );
}

#[test]
fn format_closewindow_with_cache() {
    let mut fmt = EventFormatter::new();
    let open = HyprlandEvent::parse("openwindow>>80a6f50,2,rio,rio").unwrap();
    fmt.observe(&open);
    let close = HyprlandEvent::parse("closewindow>>80a6f50").unwrap();
    // observe AFTER format for closewindow — we need the cache entry to still exist
    let msg = fmt.format(&close);
    assert_eq!(msg, "window closed (closewindow): app=rio");
    fmt.observe(&close); // now remove from cache
}

#[test]
fn format_windowtitlev2() {
    let fmt = EventFormatter::new();
    let event = HyprlandEvent::parse("windowtitlev2>>80a6f50,Yazi: ~/").unwrap();
    assert_eq!(
        fmt.format(&event),
        r#"title changed (windowtitlev2): title="Yazi: ~/""#
    );
}

#[test]
fn format_focusedmonv2() {
    let fmt = EventFormatter::new();
    let event = HyprlandEvent::parse("focusedmonv2>>DP-2,2").unwrap();
    assert_eq!(
        fmt.format(&event),
        "monitor focus (focusedmonv2): monitor=DP-2 id=2"
    );
}

#[test]
fn format_movewindowv2() {
    let fmt = EventFormatter::new();
    let event = HyprlandEvent::parse("movewindowv2>>80a6f50,2,2").unwrap();
    assert_eq!(
        fmt.format(&event),
        "window moved (movewindowv2): ws=2"
    );
}

#[test]
fn format_movewindow() {
    let fmt = EventFormatter::new();
    let event = HyprlandEvent::parse("movewindow>>80a6f50,2").unwrap();
    assert_eq!(
        fmt.format(&event),
        "window moved (movewindow): ws=2"
    );
}

#[test]
fn format_activewindow() {
    let fmt = EventFormatter::new();
    let event = HyprlandEvent::parse("activewindow>>kitty,Kitty Terminal").unwrap();
    assert_eq!(
        fmt.format(&event),
        r#"window focused (activewindow): app=kitty title="Kitty Terminal""#
    );
}

#[test]
fn format_workspace_events() {
    let fmt = EventFormatter::new();

    let event = HyprlandEvent::parse("workspace>>3").unwrap();
    assert_eq!(fmt.format(&event), "workspace changed (workspace): name=3");

    let event = HyprlandEvent::parse("createworkspace>>coding").unwrap();
    assert_eq!(
        fmt.format(&event),
        "workspace created (createworkspace): name=coding"
    );

    let event = HyprlandEvent::parse("destroyworkspace>>coding").unwrap();
    assert_eq!(
        fmt.format(&event),
        "workspace destroyed (destroyworkspace): name=coding"
    );
}

#[test]
fn format_windowtitle_with_cache() {
    let mut fmt = EventFormatter::new();
    let open = HyprlandEvent::parse("openwindow>>80a6f50,2,kitty,Kitty").unwrap();
    fmt.observe(&open);

    let event = HyprlandEvent::parse("windowtitle>>80a6f50").unwrap();
    assert_eq!(fmt.format(&event), "title changed (windowtitle): app=kitty");
}

#[test]
fn format_windowtitle_without_cache() {
    let fmt = EventFormatter::new();
    let event = HyprlandEvent::parse("windowtitle>>80a6f50").unwrap();
    assert_eq!(fmt.format(&event), "title changed (windowtitle): 80a6f50");
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --features hyprland --test hyprland_formatter -- --nocapture 2>&1 | tail -20`
Expected: New tests FAIL — `format_data()` only handles `urgent`, not the other events.

**Step 3: Implement per-event data parsing**

Expand `format_data()` in `src/hyprland/formatter.rs`:

```rust
fn format_data(&self, event: &HyprlandEvent) -> String {
    match event.name.as_str() {
        "openwindow" => self.format_openwindow(&event.data),
        "closewindow" | "urgent" | "windowtitle" => self.format_address_only(&event.data),
        "windowtitlev2" => self.format_windowtitlev2(&event.data),
        "focusedmonv2" => self.format_focusedmonv2(&event.data),
        "movewindowv2" => self.format_movewindowv2(&event.data),
        "movewindow" => self.format_movewindow(&event.data),
        "activewindow" => self.format_activewindow(&event.data),
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
        .map(String::as_str)
        .unwrap_or(class.trim());
    format!(r#"app={app} title="{title}" ws={ws}"#)
}

fn format_address_only(&self, data: &str) -> String {
    let addr = data.trim();
    match self.window_cache.get(addr) {
        Some(app) => format!("app={app}"),
        None => addr.to_string(),
    }
}

fn format_windowtitlev2(&self, data: &str) -> String {
    // Format: addr,title
    match data.split_once(',') {
        Some((_addr, title)) => format!(r#"title="{title}""#),
        None => data.to_string(),
    }
}

fn format_focusedmonv2(&self, data: &str) -> String {
    // Format: name,id
    match data.split_once(',') {
        Some((name, id)) => format!("monitor={name} id={id}"),
        None => data.to_string(),
    }
}

fn format_movewindowv2(&self, data: &str) -> String {
    // Format: addr,ws_id,ws_name
    let mut fields = data.splitn(3, ',');
    let _addr = fields.next();
    let _ws_id = fields.next();
    match fields.next() {
        Some(ws_name) => format!("ws={ws_name}"),
        None => data.to_string(),
    }
}

fn format_movewindow(&self, data: &str) -> String {
    // Format: addr,ws
    match data.split_once(',') {
        Some((_addr, ws)) => format!("ws={ws}"),
        None => data.to_string(),
    }
}

fn format_activewindow(&self, data: &str) -> String {
    // Format: class,title
    match data.split_once(',') {
        Some((class, title)) => format!(r#"app={class} title="{title}""#),
        None => data.to_string(),
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --features hyprland --test hyprland_formatter -v`
Expected: All tests PASS.

**Step 5: Run lints**

Run: `cargo clippy --features hyprland -- -D warnings`

**Step 6: Commit**

```bash
git add src/hyprland/formatter.rs tests/hyprland_formatter.rs
git commit -m "feat(hyprland): add key-value data parsing for all event types"
```

---

### Task 4: Integrate Into Listener

**Files:**
- Modify: `src/hyprland/listener.rs:122-166`
- Test: `tests/hyprland_listener.rs`

**Step 1: Update existing listener tests expectations**

The existing listener tests in `tests/hyprland_listener.rs` assert on raw message format (e.g., `"openwindow: 80a6f50,2,kitty,Kitty"`). These need to be updated to expect the new formatted output.

Update assertions in existing tests:

- `run_event_loop_logs_events_and_applies_allowlist_filter_with_app_scope` (line 100):
  Change: `assert_eq!(captured[0].message, "openwindow: 80a6f50,2,kitty,Kitty");`
  To: `assert_eq!(captured[0].message, r#"window opened (openwindow): app=kitty title="Kitty" ws=2"#);`

- `run_event_loop_custom_events_use_hyprctl_scope` (line 150):
  Change: `assert_eq!(captured[0].message, "custom: from_hyprctl");`
  To: `assert_eq!(captured[0].message, "custom: from_hyprctl");`
  (No change — unknown events keep raw format.)

- `run_event_loop_hypr_app_tokens_use_app_scope` (lines 252-255):
  Change: `assert_eq!(captured[0].message, "openwindow: 80a6f50,2,Hyprlock,Hyprlock");`
  To: `assert_eq!(captured[0].message, r#"window opened (openwindow): app=hyprlock title="Hyprlock" ws=2"#);`

- `run_event_loop_monitor_events_use_hyprland_scope_when_no_app_name` (line 305):
  Change: `assert_eq!(captured[0].message, "focusedmonv2: DP-2,2");`
  To: `assert_eq!(captured[0].message, "monitor focus (focusedmonv2): monitor=DP-2 id=2");`

**Step 2: Run tests to verify they fail**

Run: `cargo test --features hyprland --test hyprland_listener -- --nocapture 2>&1 | tail -30`
Expected: Tests FAIL — listener still uses old `event.format_message()`.

**Step 3: Integrate formatter into listener**

In `src/hyprland/listener.rs`, modify `process_events()`:

Add import at top:
```rust
use super::formatter::EventFormatter;
```

In `process_events()` function, add `let mut formatter = EventFormatter::new();` before the loop, then inside the event handling arm replace:
```rust
logger.log(level, &scope, &event.format_message());
```
with:
```rust
formatter.observe(&event);
logger.log(level, &scope, &formatter.format(&event));
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --features hyprland --test hyprland_listener -v`
Expected: All 5 listener tests PASS.

Run: `cargo test --features hyprland --test hyprland_formatter -v`
Expected: All formatter tests still PASS.

**Step 5: Run full test suite**

Run: `cargo test --features hyprland`
Expected: All tests PASS.

**Step 6: Run lints**

Run: `cargo clippy --features hyprland -- -D warnings`

**Step 7: Commit**

```bash
git add src/hyprland/listener.rs tests/hyprland_listener.rs
git commit -m "feat(hyprland): integrate EventFormatter into listener loop"
```

---

### Task 5: Closewindow Observe-After-Format Ordering

**Files:**
- Modify: `src/hyprland/listener.rs:122-166`
- Test: `tests/hyprland_formatter.rs`

The design says `closewindow` should show `app=rio` from the cache, but `observe()` removes the entry. The listener must call `format()` BEFORE `observe()` for `closewindow` — or more simply, always format first, then observe.

**Step 1: Write failing integration test**

Append to `tests/hyprland_formatter.rs`:

```rust
#[test]
fn format_then_observe_order_for_closewindow() {
    let mut fmt = EventFormatter::new();
    let open = HyprlandEvent::parse("openwindow>>80a6f50,2,rio,rio").unwrap();
    fmt.observe(&open);

    let close = HyprlandEvent::parse("closewindow>>80a6f50").unwrap();
    // The listener should format BEFORE observe, so the cache entry is still there.
    let msg = fmt.format(&close);
    assert_eq!(msg, "window closed (closewindow): app=rio");
    fmt.observe(&close);

    // After observe, cache is cleared
    let msg2 = fmt.format(&close);
    assert_eq!(msg2, "window closed (closewindow): 80a6f50");
}
```

**Step 2: Run test**

Run: `cargo test --features hyprland --test hyprland_formatter -- format_then_observe_order -v`
Expected: PASS (this tests the correct calling order).

**Step 3: Fix listener ordering**

In `src/hyprland/listener.rs`, inside `process_events()`, change the order so `format()` comes before `observe()`:

```rust
let msg = formatter.format(&event);
formatter.observe(&event);
logger.log(level, &scope, &msg);
```

**Step 4: Add listener integration test for closewindow with cache**

Append to `tests/hyprland_listener.rs`:

```rust
#[test]
fn run_event_loop_closewindow_shows_cached_app_name() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let socket_path = tmp.path().join(".socket2.sock");
    let listener_socket = UnixListener::bind(&socket_path).expect("bind socket2");

    let server_thread = thread::spawn(move || {
        let (mut stream, _) = listener_socket.accept().expect("accept socket2 client");
        writeln!(stream, "openwindow>>80a6f50,2,kitty,Kitty").expect("write openwindow");
        writeln!(stream, "closewindow>>80a6f50").expect("write closewindow");
        stream.flush().expect("flush stream");
    });

    let records = Arc::new(Mutex::new(Vec::<LogRecord>::new()));
    let logger = Logger::builder()
        .output(CaptureOutput {
            records: Arc::clone(&records),
        })
        .build();

    let config = HyprlandConfig {
        scope: "HYPRTEST".to_string(),
        ..HyprlandConfig::default()
    };

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_for_loop = Arc::clone(&shutdown);
    let socket_dir = tmp.path().to_path_buf();

    let loop_thread = thread::spawn(move || {
        listener::run_event_loop(&socket_dir, &logger, &config, &shutdown_for_loop);
    });

    assert!(
        wait_for_records(&records, 2, Duration::from_secs(3)),
        "expected at least two logged events"
    );

    thread::sleep(Duration::from_millis(100));
    shutdown.store(true, Ordering::Relaxed);

    loop_thread.join().expect("listener thread should join");
    server_thread.join().expect("server thread should join");

    let captured = records.lock().expect("records lock poisoned");
    assert_eq!(captured.len(), 2);
    assert_eq!(captured[1].message, "window closed (closewindow): app=kitty");
}
```

**Step 5: Run all tests**

Run: `cargo test --features hyprland -v`
Expected: All PASS.

**Step 6: Run lints**

Run: `cargo clippy --features hyprland -- -D warnings`

**Step 7: Commit**

```bash
git add src/hyprland/listener.rs tests/hyprland_formatter.rs tests/hyprland_listener.rs
git commit -m "fix(hyprland): format before observe so closewindow shows cached app"
```

---

### Task 6: Final Verification

**Step 1: Run full test suite**

Run: `cargo test --features hyprland`
Expected: All tests PASS.

**Step 2: Run strict lints**

Run: `cargo clippy --features hyprland -- -D warnings -W clippy::pedantic -W clippy::nursery`
Expected: Clean.

**Step 3: Check formatting**

Run: `cargo fmt --all -- --check`
Expected: Clean.

**Step 4: Update lib.rs re-export if needed**

Check if `EventFormatter` should be re-exported from `src/lib.rs`. Since it's used internally by the listener and exposed via `hyprlog::hyprland::formatter::EventFormatter` for tests, no additional re-export is needed.

**Step 5: Commit if any fixes were needed**

Only commit if lints or fmt required changes.
