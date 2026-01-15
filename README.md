<p align="center">
  <img src="assets/header.svg" alt="hyprlog" />
</p>

<p align="center">
  <strong>Unified Logging for the Hypr Ecosystem</strong>
</p>

---

## Overview

hyprlog is a structured logging system designed for the Hypr ecosystem. It provides consistent, themeable log output across CLI tools, shell scripts, and applications.

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

```
hyprlog (CLI binary)     hl_shell (interactive shell)
        └─────────────────────────┘
                    │
                 hl_core (core logging)
                    │
                hl_common (shared utilities)
```

### Crates

- **hl_core**: Core logging library with presets and formatters
- **hl_cli**: Command-line interface
- **hl_shell**: Interactive shell for log exploration
- **hl_common**: Shared CLI utilities

## License

MIT
