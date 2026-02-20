//! Tests for the Hyprland event formatter.

#![cfg(feature = "hyprland")]

use hyprlog::HyprlandEvent;
use hyprlog::hyprland::formatter::EventFormatter;

// --- Basic formatting ---

#[test]
fn format_known_event_shows_human_name_and_technical() {
    let fmt = EventFormatter::new(true);
    let event = HyprlandEvent::parse("workspace>>3").unwrap();
    assert_eq!(fmt.format(&event), "workspace changed (workspace): name=3");
}

#[test]
fn format_unknown_event_uses_technical_name() {
    let fmt = EventFormatter::new(true);
    let event = HyprlandEvent::parse("somenewevent>>payload").unwrap();
    assert_eq!(fmt.format(&event), "somenewevent: payload");
}

#[test]
fn format_empty_data_omits_colon() {
    let fmt = EventFormatter::new(true);
    let event = HyprlandEvent::parse("configreloaded>>").unwrap();
    assert_eq!(fmt.format(&event), "config reloaded");
}

#[test]
fn format_known_event_empty_data_shows_human_name() {
    let fmt = EventFormatter::new(true);
    let event = HyprlandEvent {
        name: "configreloaded".into(),
        data: String::new(),
    };
    assert_eq!(fmt.format(&event), "config reloaded");
}

// --- human_readable=false falls back to raw ---

#[test]
fn format_raw_mode_passes_through() {
    let fmt = EventFormatter::new(false);
    let event = HyprlandEvent::parse("openwindow>>80a6f50,2,kitty,Kitty").unwrap();
    assert_eq!(fmt.format(&event), "openwindow: 80a6f50,2,kitty,Kitty");
}

#[test]
fn format_raw_mode_empty_data() {
    let fmt = EventFormatter::new(false);
    let event = HyprlandEvent::parse("configreloaded>>").unwrap();
    assert_eq!(fmt.format(&event), "configreloaded");
}

// --- Window address cache ---

#[test]
fn observe_openwindow_populates_cache() {
    let mut fmt = EventFormatter::new(true);
    let open = HyprlandEvent::parse("openwindow>>80a6f50,2,kitty,Kitty").unwrap();
    fmt.observe(&open);

    let urgent = HyprlandEvent::parse("urgent>>80a6f50").unwrap();
    assert_eq!(fmt.format(&urgent), "focus requested (urgent): app=kitty");
}

#[test]
fn observe_closewindow_removes_cache_entry() {
    let mut fmt = EventFormatter::new(true);
    let open = HyprlandEvent::parse("openwindow>>80a6f50,2,kitty,Kitty").unwrap();
    fmt.observe(&open);

    let close = HyprlandEvent::parse("closewindow>>80a6f50").unwrap();
    fmt.observe(&close);

    let urgent = HyprlandEvent::parse("urgent>>80a6f50").unwrap();
    assert_eq!(fmt.format(&urgent), "focus requested (urgent): 80a6f50");
}

#[test]
fn urgent_without_cache_shows_raw_address() {
    let fmt = EventFormatter::new(true);
    let event = HyprlandEvent::parse("urgent>>80a6f50").unwrap();
    assert_eq!(fmt.format(&event), "focus requested (urgent): 80a6f50");
}

#[test]
fn format_then_observe_order_for_closewindow() {
    let mut fmt = EventFormatter::new(true);
    let open = HyprlandEvent::parse("openwindow>>80a6f50,2,rio,rio").unwrap();
    fmt.observe(&open);

    let close = HyprlandEvent::parse("closewindow>>80a6f50").unwrap();
    let msg = fmt.format(&close);
    assert_eq!(msg, "window closed (closewindow): app=rio");
    fmt.observe(&close);

    let msg2 = fmt.format(&close);
    assert_eq!(msg2, "window closed (closewindow): 80a6f50");
}

// --- Window lifecycle events ---

#[test]
fn format_openwindow_key_value() {
    let mut fmt = EventFormatter::new(true);
    let event = HyprlandEvent::parse("openwindow>>80a6f50,2,kitty,Kitty Terminal").unwrap();
    fmt.observe(&event);
    assert_eq!(
        fmt.format(&event),
        r#"window opened (openwindow): app=kitty title="Kitty Terminal" ws=2"#
    );
}

#[test]
fn format_closewindow_with_cache() {
    let mut fmt = EventFormatter::new(true);
    let open = HyprlandEvent::parse("openwindow>>80a6f50,2,rio,rio").unwrap();
    fmt.observe(&open);
    let close = HyprlandEvent::parse("closewindow>>80a6f50").unwrap();
    let msg = fmt.format(&close);
    assert_eq!(msg, "window closed (closewindow): app=rio");
    fmt.observe(&close);
}

#[test]
fn format_windowtitlev2() {
    let fmt = EventFormatter::new(true);
    let event = HyprlandEvent::parse("windowtitlev2>>80a6f50,Yazi: ~/").unwrap();
    assert_eq!(
        fmt.format(&event),
        r#"title changed (windowtitlev2): title="Yazi: ~/""#
    );
}

#[test]
fn format_windowtitle_with_cache() {
    let mut fmt = EventFormatter::new(true);
    let open = HyprlandEvent::parse("openwindow>>80a6f50,2,kitty,Kitty").unwrap();
    fmt.observe(&open);

    let event = HyprlandEvent::parse("windowtitle>>80a6f50").unwrap();
    assert_eq!(fmt.format(&event), "title changed (windowtitle): app=kitty");
}

#[test]
fn format_windowtitle_without_cache() {
    let fmt = EventFormatter::new(true);
    let event = HyprlandEvent::parse("windowtitle>>80a6f50").unwrap();
    assert_eq!(fmt.format(&event), "title changed (windowtitle): 80a6f50");
}

#[test]
fn format_activewindow() {
    let fmt = EventFormatter::new(true);
    let event = HyprlandEvent::parse("activewindow>>kitty,Kitty Terminal").unwrap();
    assert_eq!(
        fmt.format(&event),
        r#"window focused (activewindow): app=kitty title="Kitty Terminal""#
    );
}

#[test]
fn format_activewindowv2_with_cache() {
    let mut fmt = EventFormatter::new(true);
    let open = HyprlandEvent::parse("openwindow>>80a6f50,2,kitty,Kitty").unwrap();
    fmt.observe(&open);

    let event = HyprlandEvent::parse("activewindowv2>>80a6f50").unwrap();
    assert_eq!(
        fmt.format(&event),
        "window focused (activewindowv2): app=kitty"
    );
}

#[test]
fn format_movewindow() {
    let fmt = EventFormatter::new(true);
    let event = HyprlandEvent::parse("movewindow>>80a6f50,2").unwrap();
    assert_eq!(fmt.format(&event), "window moved (movewindow): ws=2");
}

#[test]
fn format_movewindowv2() {
    let fmt = EventFormatter::new(true);
    let event = HyprlandEvent::parse("movewindowv2>>80a6f50,2,2").unwrap();
    assert_eq!(fmt.format(&event), "window moved (movewindowv2): ws=2");
}

// --- Window state events ---

#[test]
fn format_fullscreen() {
    let fmt = EventFormatter::new(true);
    let event = HyprlandEvent::parse("fullscreen>>1").unwrap();
    assert_eq!(
        fmt.format(&event),
        "fullscreen toggled (fullscreen): enabled=true"
    );

    let event = HyprlandEvent::parse("fullscreen>>0").unwrap();
    assert_eq!(
        fmt.format(&event),
        "fullscreen toggled (fullscreen): enabled=false"
    );
}

#[test]
fn format_changefloatingmode_with_cache() {
    let mut fmt = EventFormatter::new(true);
    let open = HyprlandEvent::parse("openwindow>>80a6f50,2,kitty,Kitty").unwrap();
    fmt.observe(&open);

    let event = HyprlandEvent::parse("changefloatingmode>>80a6f50,1").unwrap();
    assert_eq!(
        fmt.format(&event),
        "float toggled (changefloatingmode): app=kitty tiled=true"
    );
}

#[test]
fn format_minimized() {
    let mut fmt = EventFormatter::new(true);
    let open = HyprlandEvent::parse("openwindow>>80a6f50,2,kitty,Kitty").unwrap();
    fmt.observe(&open);

    let event = HyprlandEvent::parse("minimized>>80a6f50,1").unwrap();
    assert_eq!(
        fmt.format(&event),
        "minimized (minimized): app=kitty minimized=true"
    );
}

#[test]
fn format_pin() {
    let fmt = EventFormatter::new(true);
    let event = HyprlandEvent::parse("pin>>80a6f50,1").unwrap();
    assert_eq!(fmt.format(&event), "pin toggled (pin): pinned=true");
}

// --- Workspace events ---

#[test]
fn format_workspace_events() {
    let fmt = EventFormatter::new(true);

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
fn format_workspace_v2_events() {
    let fmt = EventFormatter::new(true);

    let event = HyprlandEvent::parse("workspacev2>>3,coding").unwrap();
    assert_eq!(
        fmt.format(&event),
        "workspace changed (workspacev2): id=3 name=coding"
    );

    let event = HyprlandEvent::parse("createworkspacev2>>3,coding").unwrap();
    assert_eq!(
        fmt.format(&event),
        "workspace created (createworkspacev2): id=3 name=coding"
    );

    let event = HyprlandEvent::parse("destroyworkspacev2>>3,coding").unwrap();
    assert_eq!(
        fmt.format(&event),
        "workspace destroyed (destroyworkspacev2): id=3 name=coding"
    );
}

#[test]
fn format_moveworkspace() {
    let fmt = EventFormatter::new(true);
    let event = HyprlandEvent::parse("moveworkspace>>coding,DP-2").unwrap();
    assert_eq!(
        fmt.format(&event),
        "workspace moved (moveworkspace): name=coding monitor=DP-2"
    );
}

#[test]
fn format_moveworkspacev2() {
    let fmt = EventFormatter::new(true);
    let event = HyprlandEvent::parse("moveworkspacev2>>3,coding,DP-2").unwrap();
    assert_eq!(
        fmt.format(&event),
        "workspace moved (moveworkspacev2): id=3 name=coding monitor=DP-2"
    );
}

#[test]
fn format_renameworkspace() {
    let fmt = EventFormatter::new(true);
    let event = HyprlandEvent::parse("renameworkspace>>3,newname").unwrap();
    assert_eq!(
        fmt.format(&event),
        "workspace renamed (renameworkspace): id=3 name=newname"
    );
}

// --- Special workspace events ---

#[test]
fn format_activespecial() {
    let fmt = EventFormatter::new(true);
    let event = HyprlandEvent::parse("activespecial>>scratchpad,DP-1").unwrap();
    assert_eq!(
        fmt.format(&event),
        "special workspace (activespecial): name=scratchpad monitor=DP-1"
    );
}

#[test]
fn format_activespecialv2() {
    let fmt = EventFormatter::new(true);
    let event = HyprlandEvent::parse("activespecialv2>>5,scratchpad,DP-1").unwrap();
    assert_eq!(
        fmt.format(&event),
        "special workspace (activespecialv2): id=5 name=scratchpad monitor=DP-1"
    );
}

// --- Monitor events ---

#[test]
fn format_focusedmon() {
    let fmt = EventFormatter::new(true);
    let event = HyprlandEvent::parse("focusedmon>>DP-2,3").unwrap();
    assert_eq!(
        fmt.format(&event),
        "monitor focus (focusedmon): monitor=DP-2 workspace=3"
    );
}

#[test]
fn format_focusedmonv2() {
    let fmt = EventFormatter::new(true);
    let event = HyprlandEvent::parse("focusedmonv2>>DP-2,2").unwrap();
    assert_eq!(
        fmt.format(&event),
        "monitor focus (focusedmonv2): monitor=DP-2 id=2"
    );
}

#[test]
fn format_monitor_added_removed() {
    let fmt = EventFormatter::new(true);

    let event = HyprlandEvent::parse("monitoradded>>DP-2").unwrap();
    assert_eq!(
        fmt.format(&event),
        "monitor added (monitoradded): name=DP-2"
    );

    let event = HyprlandEvent::parse("monitorremoved>>DP-2").unwrap();
    assert_eq!(
        fmt.format(&event),
        "monitor removed (monitorremoved): name=DP-2"
    );
}

#[test]
fn format_monitoraddedv2() {
    let fmt = EventFormatter::new(true);
    let event = HyprlandEvent::parse("monitoraddedv2>>1,DP-2,Dell Inc.").unwrap();
    assert_eq!(
        fmt.format(&event),
        "monitor added (monitoraddedv2): id=1 name=DP-2 description=Dell Inc."
    );
}

// --- Group events ---

#[test]
fn format_togglegroup() {
    let fmt = EventFormatter::new(true);
    let event = HyprlandEvent::parse("togglegroup>>1,80a6f50,80a6f60").unwrap();
    assert_eq!(
        fmt.format(&event),
        "group toggled (togglegroup): state=opened windows=2"
    );

    let event = HyprlandEvent::parse("togglegroup>>0,80a6f50").unwrap();
    assert_eq!(
        fmt.format(&event),
        "group toggled (togglegroup): state=closed windows=1"
    );
}

#[test]
fn format_lockgroups() {
    let fmt = EventFormatter::new(true);
    let event = HyprlandEvent::parse("lockgroups>>1").unwrap();
    assert_eq!(
        fmt.format(&event),
        "groups locked (lockgroups): enabled=true"
    );
}

#[test]
fn format_moveintogroup() {
    let mut fmt = EventFormatter::new(true);
    let open = HyprlandEvent::parse("openwindow>>80a6f50,2,kitty,Kitty").unwrap();
    fmt.observe(&open);

    let event = HyprlandEvent::parse("moveintogroup>>80a6f50").unwrap();
    assert_eq!(
        fmt.format(&event),
        "moved into group (moveintogroup): app=kitty"
    );
}

#[test]
fn format_ignoregrouplock() {
    let fmt = EventFormatter::new(true);
    let event = HyprlandEvent::parse("ignoregrouplock>>1").unwrap();
    assert_eq!(
        fmt.format(&event),
        "group lock ignore (ignoregrouplock): enabled=true"
    );
}

// --- Layer events ---

#[test]
fn format_openlayer() {
    let fmt = EventFormatter::new(true);
    let event = HyprlandEvent::parse("openlayer>>waybar").unwrap();
    assert_eq!(
        fmt.format(&event),
        "layer opened (openlayer): namespace=waybar"
    );
}

#[test]
fn format_closelayer() {
    let fmt = EventFormatter::new(true);
    let event = HyprlandEvent::parse("closelayer>>waybar").unwrap();
    assert_eq!(
        fmt.format(&event),
        "layer closed (closelayer): namespace=waybar"
    );
}

// --- Input events ---

#[test]
fn format_activelayout() {
    let fmt = EventFormatter::new(true);
    let event = HyprlandEvent::parse("activelayout>>AT Keyboard,German").unwrap();
    assert_eq!(
        fmt.format(&event),
        "layout changed (activelayout): keyboard=AT Keyboard layout=German"
    );
}

#[test]
fn format_submap() {
    let fmt = EventFormatter::new(true);
    let event = HyprlandEvent::parse("submap>>resize").unwrap();
    assert_eq!(fmt.format(&event), "submap (submap): resize");
}

// --- Misc events ---

#[test]
fn format_screencast() {
    let fmt = EventFormatter::new(true);
    let event = HyprlandEvent::parse("screencast>>1,0").unwrap();
    assert_eq!(
        fmt.format(&event),
        "screencast (screencast): active=true owner=0"
    );
}

#[test]
fn format_bell_with_cache() {
    let mut fmt = EventFormatter::new(true);
    let open = HyprlandEvent::parse("openwindow>>80a6f50,2,kitty,Kitty").unwrap();
    fmt.observe(&open);

    let event = HyprlandEvent::parse("bell>>80a6f50").unwrap();
    assert_eq!(fmt.format(&event), "bell (bell): app=kitty");
}

#[test]
fn format_configreloaded() {
    let fmt = EventFormatter::new(true);
    let event = HyprlandEvent::parse("configreloaded>>").unwrap();
    assert_eq!(fmt.format(&event), "config reloaded");
}
