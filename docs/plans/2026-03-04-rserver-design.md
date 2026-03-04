# Design: `rserver` Feature

**Date:** 2026-03-04
**Status:** Approved

## Overview

Add a remote logging server to hyprlog, feature-gated under `rserver`. Programs on the same machine or network can send log records to a running hyprlog daemon, which handles all output (terminal, file, JSON). Clients use a Unix domain socket (local IPC) or TCP (network).

## Feature Flag

```toml
rserver = ["dep:tokio"]
```

Tokio with `features = ["net", "io-util", "rt-multi-thread", "macros", "sync"]`. `serde_json` is already a core dependency and needs no gating.

## Architecture

```
                    ┌─────────────────────────────────┐
                    │    hyprlog server daemon         │
                    │  (feature: rserver)              │
                    │                                  │
                    │  tokio::main                     │
                    │  ├── UnixListener  (/run/...)   │
                    │  ├── TcpListener  (0.0.0.0:port)│
                    │  │                               │
                    │  └── per connection task:        │
                    │       JSON-line lesen            │
                    │       → Logger.log_full()        │
                    │       → Terminal/File/JSON Out   │
                    │                                  │
                    │  Config: server.conf     │
                    └─────────────────────────────────┘
                              ▲          ▲
                    Unix      │          │  TCP
                    ┌─────────┘          └──────────┐
         ┌──────────────────┐         ┌─────────────────────┐
         │  Rust-Programm   │         │  C/C++-Programm     │
         │  RemoteOutput    │         │  FFI: hyprlog_remote │
         │  (channel+tokio) │         │  (wrapped RemoteOut) │
         └──────────────────┘         └─────────────────────┘
                    │
         ┌──────────────────┐
         │  CLI:            │
         │  hyprlog send    │
         └──────────────────┘
```

## Wire Protocol

Newline-delimited JSON. One line = one log record. Fire-and-forget (server does not respond).

```json
{"level":"info","scope":"NET","app":"myapp","message":"Connected"}
```

Fields:
- `level`: `"trace"` | `"debug"` | `"info"` | `"warn"` | `"error"`
- `scope`: component name string
- `app`: optional application name (uses server default if omitted)
- `message`: log message string

The protocol is human-readable and can be tested with `nc`:
```bash
echo '{"level":"warn","scope":"TEST","message":"hello"}' | nc -U /run/user/1000/hyprlog.sock
```

## New Modules

```
src/
  server/                    (cfg(feature = "rserver"))
    mod.rs                   — Server start, main Tokio runtime entry
    listener.rs              — UnixListener + TcpListener accept loops
    connection.rs            — JSON-line reader per connection task
    daemon.rs                — PID-file write/read, start/stop/status
    config.rs                — ServerConfig struct + TOML loading
  output/
    remote.rs                (cfg(feature = "rserver")) — RemoteOutput client
  cli/commands/
    server.rs                (cfg(feature = "rserver")) — cmd_server, cmd_send
```

## Server Config

Path: `~/.config/hypr/server.conf`

```toml
[server]
socket_path = "/run/user/1000/hyprlog.sock"
tcp_port    = 9872
tcp_bind    = "127.0.0.1"
pid_file    = "/run/user/1000/hyprlog.pid"

[output.terminal]
enabled = true
colors  = true

[output.file]
enabled = true
path    = "~/.local/share/hypr/hyprlog/server.log"
```

The server config is separate from the client config (`hyprlog.conf`) to allow independent configuration of the daemon's outputs.

## CLI Commands

Consistent with existing positional-arg pattern:

```bash
# Server lifecycle
hyprlog server start                           # Fork daemon, write PID file
hyprlog server stop                            # Send SIGTERM via PID file
hyprlog server status                          # Check if daemon is running

# Send a log to the running server
hyprlog send info SCOPE MSG                    # → default Unix socket
hyprlog send --tcp host:9872 info SCOPE MSG    # → explicit TCP
hyprlog send --app myapp info SCOPE MSG        # → with explicit app name
```

`hyprlog send` parallels the existing `hyprlog info SCOPE MSG` shorthand but routes to the server instead of local outputs.

## RemoteOutput (Rust Library API)

```rust
// Feature: rserver
let logger = Logger::builder()
    .level(Level::Debug)
    .remote()
        .socket("/run/user/1000/hyprlog.sock")  // or .tcp("127.0.0.1:9872")
        .done()
    .build();

logger.info("NET", "Connected"); // fire-and-forget via internal channel
```

### Internals

`RemoteOutput` holds a `tokio::sync::mpsc::Sender<LogRecord>`. On construction, a dedicated `tokio::runtime::Runtime` is created in a background OS thread. A Tokio task in that runtime drains the channel and writes JSON lines to the server socket. The `Output::write()` method on `RemoteOutput` is therefore non-blocking from the caller's perspective — it just enqueues the record.

Dropped records (if channel is full) are silently discarded to avoid slowing down the calling program. Channel capacity: configurable, default 1024.

## FFI

```c
// Feature: rserver + ffi
HyprlogContext* ctx = hyprlog_init_remote("/run/user/1000/hyprlog.sock");
hyprlog_info(ctx, "NET", "Connected");
hyprlog_free(ctx);
```

`hyprlog_init_remote` creates a `Logger` with a `RemoteOutput` pointing to the given Unix socket path.

## Error Handling

- **Server not running when client connects:** `RemoteOutput` silently drops the record and logs internally via `internal::warn`. No panic.
- **Server startup failure** (socket in use, port in use): `cmd_server_start` returns an error and exits non-zero.
- **Malformed JSON on server side:** Server logs the parse error via `internal::warn` and continues (no connection close).

## Testing

- Integration test: start server in background thread, send via `RemoteOutput`, check server received and logged.
- CLI test: `hyprlog server start` → `hyprlog send info TEST "hello"` → `hyprlog server stop`.
- Protocol test: connect via `nc`, send raw JSON, check server output.

## Non-Goals (YAGNI)

- Authentication/TLS: out of scope for initial implementation
- Message acknowledgement / at-least-once delivery
- Log buffering/replay if server is down
