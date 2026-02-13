# hyprlog

Unified logging for the Hypr ecosystem. CLI tool, Rust library, C-ABI FFI, and optional Hyprland IPC event streaming.

## Install

```bash
# from source
./install.sh

# remote
curl -fsSL https://raw.githubusercontent.com/ryugen-io/hyprlog/main/install.sh | bash
```

Installs to `~/.local/bin/hypr/hyprlog`. Config at `~/.config/hypr/hyprlog.conf`.

## Usage

### CLI

```bash
hyprlog info INIT "Application started"
hyprlog myapp info NET "Connection established"
hyprlog log myapp error NET "Connection failed"
echo '{"level":"info","scope":"TEST","msg":"hello"}' | hyprlog json
hyprlog preset startup
hyprlog stats
hyprlog cleanup --dry-run
hyprlog cleanup --compress --older-than 7d --keep-last 5
hyprlog themes preview
hyprlog watch                              # stream Hyprland events
hyprlog watch --events openwindow,closewindow --min-level warn
hyprlog                                    # interactive shell
```

### Rust Library

```rust
use hyprlog::{Logger, Level};

let logger = Logger::builder()
    .level(Level::Debug)
    .terminal()
        .colors(true)
        .done()
    .file()
        .base_dir("~/.local/state/hyprlog/logs")
        .done()
    .json()
        .path("~/.local/state/hyprlog/db/hyprlog.jsonl")
        .done()
    .build();

logger.info("MAIN", "Application started");
logger.warn("NET", "Connection <bold>timeout</bold>");
logger.error("NET", "Connection <red>failed</red>");
```

### C-ABI (FFI)

```c
#include "hyprlog.h"

HyprlogContext* ctx = hyprlog_init_simple();
hyprlog_info(ctx, "MAIN", "Hello from C");
hyprlog_free(ctx);
```

Header: `include/hyprlog.h`. Link against `libhyprlog.so` / `libhyprlog.a`. See `examples/cpp/` for a full CMake example.

## Configuration

TOML format at `~/.config/hypr/hyprlog.conf`. Supports Hyprland-style `source = "path"` includes with cycle detection.

```toml
[general]
level = "info"

[terminal]
enabled = true
colors = true
icons = "nerdfont"           # nerdfont | ascii | none
structure = "{tag} {scope}  {msg}"

[file]
enabled = true
base_dir = "~/.local/state/hyprlog/logs"

[json]
enabled = false
path = "~/.local/state/hyprlog/db/hyprlog.jsonl"

[cleanup]
max_age_days = 30
max_total_size = "500M"
keep_last = 5

[hyprland]
enabled = true
scope = "HYPR"
ignore_events = ["mousemove"]

[tag]
transform = "uppercase"
alignment = "center"
width = 7

[scope]
transform = "uppercase"
width = 10

[highlight]
enabled = true
urls = true
paths = true
numbers = true
quotes = true

[colors]
accent = "#89b4fa"
success = "#a6e3a1"

[presets.startup]
level = "info"
scope = "INIT"
msg = "Application started"
app_name = "myapp"

[apps.myapp]
level = "debug"
[apps.myapp.terminal]
colors = true
icons = "ascii"
```

## Formatting

Messages support inline styling with XML-like tags:

```
logger.info("MAIN", "<bold>Server</bold> listening on <cyan>:8080</cyan>");
logger.error("NET", "<red>Connection failed</red>: <dim>timeout after 30s</dim>");
```

Available tags: `<bold>`, `<dim>`, `<italic>`, `<underline>`, `<red>`, `<green>`, `<yellow>`, `<cyan>`, `<blue>`, `<purple>`, `<pink>`, `<orange>`, `<white>`, and custom colors from config.

Auto-highlighting detects URLs, file paths, numbers, and quoted strings without manual tagging.

Output template placeholders: `{tag}`, `{icon}`, `{scope}`, `{msg}`, `{level}`, `{app}`, `{timestamp}`.

## Architecture

Single crate, feature-gated modules:

```
src/
  bin/hyprlog.rs       CLI entry point
  lib.rs               Library entry point
  error.rs             Unified error type
  logger/              Logger + builder pattern
  output/              Terminal, File, JSON backends (trait Output)
  config/              TOML config with source includes
  fmt/                 Formatting: color, style, tags, scope, icons, highlight, templates
  level/               Log levels (Trace, Debug, Info, Warn, Error)
  cleanup/             Age/size-based log cleanup with gzip compression
  internal/            Internal hyprlog logger (OnceLock)
  cli/                 CLI commands (feature: cli)
  shell/               Interactive REPL with themes (feature: cli)
  hyprland/            Hyprland socket2 event listener (feature: hyprland)
  ffi.rs               C-ABI bindings (feature: ffi)
```

### Features

| Feature    | Default | Description                              |
|------------|---------|------------------------------------------|
| `cli`      | yes     | CLI binary and interactive shell         |
| `ffi`      |         | C-ABI bindings (`libhyprlog.so`)         |
| `hyprland` |         | Hyprland IPC event streaming             |

## Development

Requires Rust edition 2024. Uses `just` as task runner.

```bash
just build            # release build
just build-debug      # debug build
just test             # cargo-nextest (fallback: cargo test)
just fmt              # cargo fmt
just lint             # clippy -D warnings
just bench            # criterion benchmarks
just fuzz             # cargo-fuzz (all 6 targets, 30s each)
just docs             # rustdoc
just pre-commit       # fmt + lint + test
just clean            # cargo clean
just size             # binary size
just bloat            # cargo-bloat
just audit            # cargo-audit
just outdated         # cargo-outdated
just coverage         # tarpaulin coverage
just loc              # lines of code
just tree             # source tree
```

Direct cargo:

```bash
cargo test --features hyprland          # all tests with hyprland
cargo test --test config                # single test file
cargo test --test config -- test_name   # single test
cargo test --lib                        # unit tests only
cargo bench                             # criterion benchmarks
```

### Test Suite

145 tests across 26 integration test files and 3 unit test modules. Uses `tempfile` for filesystem tests.

### Benchmarks

3 criterion benchmark files: parsing, formatting, output. Reports at `target/criterion/report/index.html`.

### Fuzz Testing

6 fuzz targets via `cargo-fuzz` (requires nightly):

- `fuzz_color_hex` - hex color parsing
- `fuzz_config_sources` - config source extraction
- `fuzz_event_parse` - Hyprland event parsing
- `fuzz_format_template` - template parsing
- `fuzz_highlight` - auto-highlight injection
- `fuzz_style_parse` - inline style tag parsing

## License

MIT
