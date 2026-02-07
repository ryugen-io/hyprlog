<p align="center">
  <img src="assets/header.svg" alt="hyprlog" />
</p>

<p align="center">
  <strong>Unified Logging for the Hypr Ecosystem</strong>
</p>

---

## Overview

hyprlog is a logging library and CLI tool for the Hypr ecosystem, written in Rust. Single crate with feature-gated modules for CLI, C-ABI FFI, and Hyprland IPC.

- Terminal, file, and JSON output backends
- Inline styling (`<bold>`, `<red>`) and auto-highlighting (URLs, paths, numbers)
- TOML config with `source = "..."` includes
- Interactive shell
- Hyprland IPC event listener and command dispatch

### Hyprland Integration

`hyprlog watch` connects to the Hyprland compositor via IPC and logs events (workspace switches, window opens, etc.) in real time. Add it to your Hyprland config to run on startup:

```ini
# ~/.config/hypr/hyprland.conf
exec-once = hyprlog watch
```

## Installation

### Option A: One-liner (Recommended)
```bash
curl -fsSL https://raw.githubusercontent.com/ryugen-io/hyprlog/master/install.sh | bash
```

### Option B: From Source
```bash
git clone https://github.com/ryugen-io/hyprlog.git
cd hyprlog
just install
```

## Configuration

Config file: `~/.config/hypr/hyprlog.conf`

### Splitting Config (Hyprland-Style)

```toml
# Source other config files
source = "~/.config/hypr/hyprlog.d/colors.conf"
source = "~/.config/hypr/hyprlog.d/presets.conf"

[general]
level = "info"
```

### Full Example

```toml
[general]
level = "info"                    # debug, info, warn, error
app_name = "hyprlog"

[terminal]
enabled = true
colors = true
icons = "nerdfont"                # nerdfont, ascii, none
structure = "{tag} {scope}  {msg}"

[file]
enabled = false
base_dir = "~/.local/state/hyprlog/logs"

[tag]
prefix = "["
suffix = "]"
transform = "uppercase"           # none, uppercase, lowercase, capitalize

# Presets/Dictionary
[presets.startup]
level = "info"
scope = "INIT"
msg = "Application started"
```

## Usage

```bash
# Basic logging
hyprlog info "Application started"
hyprlog warn "Configuration missing"
hyprlog error "Connection failed"

# With scope
hyprlog --scope "MODULE" info "Initialized"

# Use preset
hyprlog --preset startup
```

## Architecture

Single crate with feature-gated modules:

```
hyprlog
├── logger/       Core logging engine (builder pattern, output dispatch)
├── output/       Output backends (terminal, file, JSON)
├── config/       TOML config with source includes and per-app overrides
├── fmt/          Formatting (color, style tags, templates, highlighting)
├── level/        Log levels (Trace → Error)
├── cleanup/      Log rotation (age/size/compression)
├── cli/          CLI subcommands [feature: cli]
├── shell/        Interactive REPL [feature: cli]
├── hyprland/     Hyprland IPC integration [feature: hyprland]
└── ffi.rs        C-ABI bindings [feature: ffi]
```

## Development

```bash
just test               # Run test suite
just lint               # Clippy
just bench              # Criterion benchmarks
just fuzz               # Fuzz all targets (30s each)
```

## License

MIT
