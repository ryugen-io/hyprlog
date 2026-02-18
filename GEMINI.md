# hyprlog

**Unified Logging for the Hypr Ecosystem**

## Project Overview

`hyprlog` is a Rust-based logging library and CLI tool designed for the Hypr ecosystem (Hyprland). It provides a flexible and unified logging solution with features like:

*   **Multiple Backends:** Supports terminal (with colors/icons), file (with rotation/retention), and JSON database outputs.
*   **Rich Formatting:** Inline styling (e.g., `<bold>`, `<red>`) and automatic highlighting of URLs, paths, and numbers.
*   **Configuration:** TOML-based configuration (`~/.config/hypr/hyprlog.conf`) with support for including other config files (`source = "..."`).
*   **Hyprland Integration:** Can connect to Hyprland via IPC to watch and log events.
*   **Interactive Shell:** Includes a REPL for interactive use with themes and history.
*   **FFI:** C-ABI bindings for integration with C/C++ projects.
*   **Builder Pattern:** Fluent API for programmatic configuration in Rust.

## Architecture

The project is structured as a single crate with feature-gated modules:

### Source Structure (`src/`)

*   **`bin/hyprlog.rs`**: The main entry point for the CLI application.
*   **`lib.rs`**: The library entry point, exporting the public API.
*   **`logger/`**: Core logging engine implementing the builder pattern (`Logger`, `LoggerBuilder`).
*   **`output/`**: Output backend implementations:
    *   `terminal.rs`: ANSI-colored output with Nerd Font support.
    *   `file.rs`: File logging with rotation and retention policies.
    *   `json.rs`: JSONL structured logging.
*   **`config/`**: Configuration parsing and management. `structs.rs` defines the TOML schema.
*   **`fmt/`**: Formatting logic:
    *   `style.rs`: XML-like tag parsing (`<bold>`).
    *   `highlight.rs`: Regex-based highlighting (URLs, paths).
    *   `color.rs`: ANSI color handling.
*   **`hyprland/`**: Hyprland IPC event listener and integration (`socket2`).
*   **`cli/`**: CLI command definitions and logic (using `clap`).
*   **`shell/`**: Interactive shell implementation (using `rustyline`).
*   **`ffi.rs`**: C-ABI bindings (`hyprlog_init`, `hyprlog_log`).

### Configuration

The default configuration is located at `~/.config/hypr/hyprlog.conf`.

**Key Configuration Sections:**

*   **`[general]`**: `level` (min log level), `app_name`.
*   **`[terminal]`**: `enabled`, `colors`, `icons`, `structure` (template).
*   **`[file]`**: `enabled`, `base_dir`, `retention` (days/size), `filename_structure`.
*   **`[json]`**: `enabled`, `path` (JSONL db).
*   **`[hyprland]`**: `enabled`, `event_levels`, `ignore_events`.
*   **`[presets]`**: Dictionary of pre-defined log messages.

## Development

This project uses `just` as a command runner, which delegates to scripts in `dev/scripts/`.

### Prerequisites

*   Rust (1.85+)
*   `cargo-nextest` (optional, recommended for testing)
*   `just` (command runner)

### Key Commands

*   **Build:**
    *   `just build` (Release mode): Wraps `./dev/scripts/build/build.sh --release`
    *   `just build-debug` (Debug mode): Wraps `./dev/scripts/build/build.sh`
*   **Test:**
    *   `just test`: Runs tests using `cargo-nextest` if available, or `cargo test`. Wraps `./dev/scripts/test/quick.sh`.
*   **Code Quality:**
    *   `just lint`: Runs `cargo clippy`. Wraps `./dev/scripts/code/lint.sh`.
    *   `just fmt`: Runs `cargo fmt`. Wraps `./dev/scripts/code/fmt.sh`.
*   **Benchmarks:** `just bench`
*   **Install:** `just install` (Runs `./install.sh`)

### FFI / C-Binding

The `ffi` feature enables C-compatible symbols.
*   **Headers:** `include/hyprlog.h`
*   **Usage:** Link against `libhyprlog.so` / `libhyprlog.a`.

## Dependencies

*   **Core:** `serde`, `toml`, `serde_json` (Config/Data), `chrono` (Time), `regex` (Highlighting), `ulid` (IDs).
*   **CLI:** `clap` (Args), `rustyline` (Shell).
*   **System:** `directories` (Paths).
