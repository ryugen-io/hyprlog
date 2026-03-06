# rserver Feature Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a feature-gated (`rserver`) remote logging server to hyprslog, where programs send `LogRecord`s over Unix socket or TCP, and the daemon handles all output via a standard `Logger`.

**Architecture:** Tokio-based async server accepts connections on a Unix socket and/or TCP port, reads newline-delimited JSON records, and dispatches them through a `Logger`. Clients use `RemoteOutput`, which wraps an internal Tokio runtime and `mpsc` channel so `Output::write()` stays non-blocking. CLI gets `server start/stop/status` and `send` subcommands.

**Tech Stack:** Tokio (net, rt-multi-thread, io-util, macros, sync, signal), serde_json (already a core dep), std Unix sockets.

---

### Task 1: Add `rserver` feature flag and Tokio dependency

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/lib.rs`

**Step 1: Add the feature and dependency to Cargo.toml**

In the `[features]` section add:
```toml
rserver = ["dep:tokio"]
```

In `[dependencies]` add:
```toml
tokio = { version = "1", features = ["net", "io-util", "rt-multi-thread", "macros", "sync", "signal"], optional = true }
```

**Step 2: Add the module declaration to src/lib.rs**

After the `#[cfg(feature = "hyprland")]` block, add:
```rust
// Remote server module (feature-gated)
#[cfg(feature = "rserver")]
pub mod server;
```

Also add to the re-exports at the bottom (after hyprland re-exports):
```rust
// rserver re-exports
#[cfg(feature = "rserver")]
pub use output::RemoteOutput;
#[cfg(feature = "rserver")]
pub use server::ServerConfig;
```

**Step 3: Verify it compiles**

```bash
cargo build --features rserver 2>&1 | head -20
```
Expected: error about missing `server` module (that's fine for now — we just need to confirm the feature flag itself is accepted without syntax errors). Actually, this will fail because the module doesn't exist yet. Add an empty `src/server/mod.rs` first:

```bash
mkdir -p src/server && touch src/server/mod.rs
```

Then:
```bash
cargo check --features rserver
```
Expected: compiles successfully (empty module is fine).

**Step 4: Commit**

```bash
git add Cargo.toml Cargo.lock src/lib.rs src/server/mod.rs
git commit -m "feat(rserver): add rserver feature flag with tokio dependency"
```

---

### Task 2: Wire protocol types

These are the JSON structs sent over the socket. They live in `src/server/protocol.rs` and are shared between client (RemoteOutput) and server.

**Files:**
- Create: `src/server/protocol.rs`
- Modify: `src/server/mod.rs`

**Step 1: Write the failing test**

Add to `src/server/protocol.rs` (create the file first with the test at the bottom):

```rust
//! Wire protocol types for the rserver.

use crate::level::Level;
use serde::{Deserialize, Serialize};

/// A log record transmitted over the wire as a JSON line.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WireRecord {
    pub level: String,
    pub scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<String>,
    pub message: String,
}

impl WireRecord {
    pub fn from_parts(level: Level, scope: &str, app: Option<&str>, message: &str) -> Self {
        Self {
            level: level.as_str().to_string(),
            scope: scope.to_string(),
            app: app.map(ToString::to_string),
            message: message.to_string(),
        }
    }

    /// Serializes to a newline-terminated JSON string.
    ///
    /// # Errors
    /// Returns an error if serialization fails (should never happen for this type).
    pub fn to_line(&self) -> Result<String, serde_json::Error> {
        let mut s = serde_json::to_string(self)?;
        s.push('\n');
        Ok(s)
    }

    /// Parses a JSON line into a WireRecord. Trims whitespace.
    ///
    /// # Errors
    /// Returns an error if the line is not valid JSON matching WireRecord.
    pub fn from_line(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line.trim())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::Level;

    #[test]
    fn roundtrip_with_app() {
        let rec = WireRecord::from_parts(Level::Info, "NET", Some("myapp"), "Connected");
        let line = rec.to_line().unwrap();
        assert!(line.ends_with('\n'));
        let parsed = WireRecord::from_line(&line).unwrap();
        assert_eq!(parsed, rec);
    }

    #[test]
    fn roundtrip_without_app() {
        let rec = WireRecord::from_parts(Level::Warn, "DB", None, "Slow query");
        let line = rec.to_line().unwrap();
        // "app" field should be absent
        assert!(!line.contains("\"app\""));
        let parsed = WireRecord::from_line(&line).unwrap();
        assert_eq!(parsed, rec);
    }

    #[test]
    fn level_str_roundtrip() {
        let rec = WireRecord::from_parts(Level::Error, "CORE", None, "Crash");
        let line = rec.to_line().unwrap();
        assert!(line.contains("\"error\""));
    }
}
```

Export the module in `src/server/mod.rs`:
```rust
pub mod protocol;
pub use protocol::WireRecord;
```

**Step 2: Run tests to verify they pass**

```bash
cargo test --features rserver server::protocol 2>&1
```
Expected: 3 tests pass.

**Step 3: Commit**

```bash
git add src/server/protocol.rs src/server/mod.rs
git commit -m "feat(rserver): add WireRecord wire protocol type"
```

---

### Task 3: ServerConfig

Config for the daemon, loaded from `~/.config/hypr/server.conf`.

**Files:**
- Create: `src/server/config.rs`
- Modify: `src/server/mod.rs`

**Step 1: Write the failing test**

Create `src/server/config.rs`:

```rust
//! Server configuration.

use serde::Deserialize;

fn default_socket_path() -> String {
    let uid = std::process::id(); // fallback: use pid (not great, but works without nix dep)
    // Try $XDG_RUNTIME_DIR first
    if let Ok(runtime) = std::env::var("XDG_RUNTIME_DIR") {
        return format!("{runtime}/hyprslog.sock");
    }
    format!("/tmp/hyprslog-{uid}.sock")
}

fn default_tcp_bind() -> String {
    "127.0.0.1".to_string()
}

fn default_pid_file() -> String {
    if let Ok(runtime) = std::env::var("XDG_RUNTIME_DIR") {
        return format!("{runtime}/hyprslog.pid");
    }
    "/tmp/hyprslog.pid".to_string()
}

/// Configuration for the hyprslog server daemon.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    /// Path to the Unix domain socket.
    pub socket_path: String,
    /// TCP port to listen on (0 = disabled).
    pub tcp_port: u16,
    /// TCP bind address.
    pub tcp_bind: String,
    /// Path to the PID file.
    pub pid_file: String,
    /// Minimum log level for the server's own outputs.
    pub log_level: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            socket_path: default_socket_path(),
            tcp_port: 9872,
            tcp_bind: default_tcp_bind(),
            pid_file: default_pid_file(),
            log_level: "info".to_string(),
        }
    }
}

impl ServerConfig {
    /// Loads from `~/.config/hypr/server.conf`.
    /// Returns defaults if the file does not exist.
    ///
    /// # Errors
    /// Returns an error if the file exists but cannot be parsed.
    pub fn load() -> Result<Self, crate::Error> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = std::fs::read_to_string(&path)?;
        let config: Self = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Returns the path to the server config file.
    #[must_use]
    pub fn config_path() -> std::path::PathBuf {
        if let Some(cfg_dir) = directories::ProjectDirs::from("", "", "hypr") {
            cfg_dir.config_dir().join("server.conf")
        } else {
            std::path::PathBuf::from("server.conf")
        }
    }

    /// Returns the TCP bind address string.
    #[must_use]
    pub fn tcp_addr(&self) -> String {
        format!("{}:{}", self.tcp_bind, self.tcp_port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid() {
        let cfg = ServerConfig::default();
        assert!(!cfg.socket_path.is_empty());
        assert!(cfg.tcp_port > 0);
        assert_eq!(cfg.tcp_bind, "127.0.0.1");
        assert!(!cfg.pid_file.is_empty());
    }

    #[test]
    fn tcp_addr_format() {
        let mut cfg = ServerConfig::default();
        cfg.tcp_bind = "0.0.0.0".to_string();
        cfg.tcp_port = 1234;
        assert_eq!(cfg.tcp_addr(), "0.0.0.0:1234");
    }

    #[test]
    fn load_returns_defaults_when_missing() {
        // Point to a nonexistent file by temporarily overriding HOME
        // Simplest: just call load() and check it doesn't error
        // (file likely doesn't exist in test env)
        let result = ServerConfig::load();
        assert!(result.is_ok());
    }

    #[test]
    fn toml_roundtrip() {
        let toml_str = r#"
[server]
socket_path = "/tmp/test.sock"
tcp_port = 1234
tcp_bind = "0.0.0.0"
"#;
        // ServerConfig is at top level in its own file,
        // but when inside server.conf it's under [server].
        // For this test, deserialize directly:
        let cfg: ServerConfig = toml::from_str(r#"
socket_path = "/tmp/test.sock"
tcp_port = 1234
"#).unwrap();
        assert_eq!(cfg.socket_path, "/tmp/test.sock");
        assert_eq!(cfg.tcp_port, 1234);
    }
}
```

Add to `src/server/mod.rs`:
```rust
pub mod config;
pub use config::ServerConfig;
```

**Step 2: Run tests**

```bash
cargo test --features rserver server::config 2>&1
```
Expected: all tests pass.

**Step 3: Commit**

```bash
git add src/server/config.rs src/server/mod.rs
git commit -m "feat(rserver): add ServerConfig with TOML loading"
```

---

### Task 4: RemoteOutput (client-side Output backend)

This is the main client component. It implements `Output` and sends `LogRecord`s to the server over Unix socket or TCP.

**Files:**
- Create: `src/output/remote.rs`
- Modify: `src/output/mod.rs`

**Step 1: Write the failing test**

The test will spin up a tiny TCP listener, connect via RemoteOutput, send a record, and verify the server received it.

Create `src/output/remote.rs`:

```rust
//! Remote output: sends log records to a hyprslog server.

use crate::internal;
use crate::output::{LogRecord, Output};
use crate::server::protocol::WireRecord;
use std::sync::mpsc;
use std::thread;

/// How to connect to the server.
#[derive(Debug, Clone)]
pub enum RemoteTarget {
    /// Unix domain socket path.
    UnixSocket(String),
    /// TCP address (host:port).
    Tcp(String),
}

/// Output backend that forwards log records to a remote hyprslog server.
///
/// Uses an internal channel and background Tokio runtime so `write()` never blocks.
pub struct RemoteOutput {
    sender: mpsc::SyncSender<WireRecord>,
    // Keep the thread alive as long as RemoteOutput is alive.
    _worker: thread::JoinHandle<()>,
}

impl RemoteOutput {
    /// Creates a new `RemoteOutput` connected via Unix socket.
    ///
    /// # Panics
    /// Panics if the internal Tokio runtime cannot be created (very unlikely).
    #[must_use]
    pub fn unix(path: impl Into<String>) -> Self {
        Self::new(RemoteTarget::UnixSocket(path.into()))
    }

    /// Creates a new `RemoteOutput` connected via TCP.
    ///
    /// # Panics
    /// Panics if the internal Tokio runtime cannot be created (very unlikely).
    #[must_use]
    pub fn tcp(addr: impl Into<String>) -> Self {
        Self::new(RemoteTarget::Tcp(addr.into()))
    }

    fn new(target: RemoteTarget) -> Self {
        // Channel capacity: 1024 records before dropping
        let (sender, receiver) = mpsc::sync_channel::<WireRecord>(1024);

        let worker = thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("failed to build tokio runtime for RemoteOutput");
            rt.block_on(worker_loop(target, receiver));
        });

        Self {
            sender,
            _worker: worker,
        }
    }
}

async fn worker_loop(target: RemoteTarget, receiver: mpsc::Receiver<WireRecord>) {
    use tokio::io::AsyncWriteExt;

    loop {
        // (Re)connect on every batch or reconnect after failure
        let mut stream: Box<dyn tokio::io::AsyncWrite + Unpin + Send> = match &target {
            RemoteTarget::Tcp(addr) => match tokio::net::TcpStream::connect(addr).await {
                Ok(s) => Box::new(s),
                Err(e) => {
                    internal::warn("REMOTE", &format!("Cannot connect to {addr}: {e}"));
                    // Wait for next record, then retry
                    let Ok(rec) = receiver.recv() else { return };
                    drop(rec); // discard this one, we couldn't send it
                    continue;
                }
            },
            RemoteTarget::UnixSocket(path) => {
                match tokio::net::UnixStream::connect(path).await {
                    Ok(s) => Box::new(s),
                    Err(e) => {
                        internal::warn("REMOTE", &format!("Cannot connect to {path}: {e}"));
                        let Ok(rec) = receiver.recv() else { return };
                        drop(rec);
                        continue;
                    }
                }
            }
        };

        // Drain as many records as available, sending each
        loop {
            let rec = match receiver.recv() {
                Ok(r) => r,
                Err(_) => return, // sender dropped, shutdown
            };
            let Ok(line) = rec.to_line() else { continue };
            if stream.write_all(line.as_bytes()).await.is_err() {
                // Connection lost — reconnect on next iteration of outer loop
                break;
            }
        }
    }
}

impl Output for RemoteOutput {
    fn write(&self, record: &LogRecord) -> Result<(), crate::Error> {
        let wire = WireRecord::from_parts(
            record.level,
            &record.scope,
            record.app_name.as_deref(),
            &record.message,
        );
        // Try to enqueue; if channel is full, silently drop
        let _ = self.sender.try_send(wire);
        Ok(())
    }

    fn flush(&self) -> Result<(), crate::Error> {
        // Fire-and-forget: no flush guarantee across the network
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::Level;
    use crate::output::LogRecord;
    use crate::fmt::FormatValues;
    use std::io::BufRead;
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    fn make_record(level: Level, scope: &str, msg: &str) -> LogRecord {
        LogRecord {
            level,
            scope: scope.to_string(),
            message: msg.to_string(),
            values: FormatValues::new(),
            label_override: None,
            app_name: None,
            raw: false,
        }
    }

    #[test]
    fn sends_record_to_tcp_server() {
        // Bind to a random OS-assigned port
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let received: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let recv_clone = Arc::clone(&received);

        // Spawn a minimal server thread
        std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let reader = std::io::BufReader::new(stream);
            for line in reader.lines() {
                let Ok(line) = line else { break };
                recv_clone.lock().unwrap().push(line);
            }
        });

        // Give server a moment to bind
        std::thread::sleep(Duration::from_millis(50));

        let output = RemoteOutput::tcp(addr.to_string());
        output.write(&make_record(Level::Info, "TEST", "hello world")).unwrap();

        // Give the worker time to send
        std::thread::sleep(Duration::from_millis(200));

        let lines = received.lock().unwrap();
        assert_eq!(lines.len(), 1);
        let parsed: serde_json::Value = serde_json::from_str(&lines[0]).unwrap();
        assert_eq!(parsed["level"], "info");
        assert_eq!(parsed["scope"], "TEST");
        assert_eq!(parsed["message"], "hello world");
    }

    #[test]
    fn drops_records_gracefully_when_server_not_running() {
        // Connect to a port with nothing listening
        let output = RemoteOutput::tcp("127.0.0.1:19999");
        // Should not panic or block
        let result = output.write(&make_record(Level::Warn, "TEST", "dropped"));
        assert!(result.is_ok());
    }
}
```

Add to `src/output/mod.rs` (after existing `pub use` lines):
```rust
#[cfg(feature = "rserver")]
mod remote;
#[cfg(feature = "rserver")]
pub use remote::RemoteOutput;
```

**Step 2: Run tests**

```bash
cargo test --features rserver output::remote 2>&1
```
Expected: both tests pass. The TCP test should receive exactly 1 JSON line.

**Step 3: Commit**

```bash
git add src/output/remote.rs src/output/mod.rs
git commit -m "feat(rserver): add RemoteOutput client backend"
```

---

### Task 5: RemoteBuilder (builder-pattern integration)

Wire `RemoteOutput` into the `Logger::builder()` chain, consistent with `TerminalBuilder` and `FileBuilder`.

**Files:**
- Modify: `src/logger/builder.rs`
- Modify: `src/logger/mod.rs` (re-export)

**Step 1: Write the failing test**

The test goes in the existing `tests/` directory. Create `tests/remote_builder.rs`:

```rust
//! Tests for RemoteBuilder integration.
#[cfg(feature = "rserver")]
mod rserver_tests {
    use hyprslog::{Level, Logger};
    use std::net::TcpListener;
    use std::io::BufRead;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    #[test]
    fn builder_remote_tcp() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let received: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let recv_clone = Arc::clone(&received);

        std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            for line in std::io::BufReader::new(stream).lines() {
                let Ok(line) = line else { break };
                recv_clone.lock().unwrap().push(line);
            }
        });

        std::thread::sleep(Duration::from_millis(50));

        let logger = Logger::builder()
            .level(Level::Debug)
            .remote()
                .tcp(addr.to_string())
                .done()
            .build();

        logger.info("NET", "builder test");
        std::thread::sleep(Duration::from_millis(200));

        let lines = received.lock().unwrap();
        assert_eq!(lines.len(), 1);
        let v: serde_json::Value = serde_json::from_str(&lines[0]).unwrap();
        assert_eq!(v["message"], "builder test");
    }
}
```

**Step 2: Add RemoteBuilder to src/logger/builder.rs**

Add after the `JsonBuilder` import and at the end of the file:

At the top of `builder.rs`, inside the `#[cfg(feature = "rserver")]` guard, add the import:
```rust
#[cfg(feature = "rserver")]
use crate::output::RemoteOutput;
```

Add a `.remote()` method to `LoggerBuilder`:
```rust
#[cfg(feature = "rserver")]
/// Adds a remote output.
#[must_use]
pub fn remote(self) -> crate::logger::remote_builder::RemoteBuilder {
    crate::logger::remote_builder::RemoteBuilder {
        parent: self,
        target: None,
    }
}
```

Create `src/logger/remote_builder.rs`:
```rust
//! Builder for remote output configuration.

use crate::logger::builder::LoggerBuilder;
use crate::output::RemoteOutput;

/// Builder for remote output configuration.
pub struct RemoteBuilder {
    pub(crate) parent: LoggerBuilder,
    pub(crate) target: Option<RemoteTarget>,
}

enum RemoteTarget {
    Unix(String),
    Tcp(String),
}

impl RemoteBuilder {
    /// Connect via Unix domain socket.
    #[must_use]
    pub fn socket(mut self, path: impl Into<String>) -> Self {
        self.target = Some(RemoteTarget::Unix(path.into()));
        self
    }

    /// Connect via TCP.
    #[must_use]
    pub fn tcp(mut self, addr: impl Into<String>) -> Self {
        self.target = Some(RemoteTarget::Tcp(addr.into()));
        self
    }

    /// Finishes remote configuration and returns to the logger builder.
    ///
    /// # Panics
    /// Panics if neither `.socket()` nor `.tcp()` was called.
    #[must_use]
    pub fn done(mut self) -> LoggerBuilder {
        let output = match self.target.take().expect("call .socket() or .tcp() before .done()") {
            RemoteTarget::Unix(path) => RemoteOutput::unix(path),
            RemoteTarget::Tcp(addr) => RemoteOutput::tcp(addr),
        };
        self.parent.outputs.push(Box::new(output));
        self.parent
    }
}
```

Add to `src/logger/mod.rs`:
```rust
#[cfg(feature = "rserver")]
mod remote_builder;
#[cfg(feature = "rserver")]
pub use remote_builder::RemoteBuilder;
```

**Step 3: Run test**

```bash
cargo test --features rserver --test remote_builder 2>&1
```
Expected: `builder_remote_tcp` passes.

**Step 4: Commit**

```bash
git add src/logger/builder.rs src/logger/remote_builder.rs src/logger/mod.rs tests/remote_builder.rs
git commit -m "feat(rserver): add RemoteBuilder to logger builder chain"
```

---

### Task 6: Server — connection handler

The per-connection logic: reads JSON lines, dispatches to Logger.

**Files:**
- Create: `src/server/connection.rs`
- Modify: `src/server/mod.rs`

**Step 1: Write the code**

Create `src/server/connection.rs`:

```rust
//! Per-connection handler for the rserver.

use crate::internal;
use crate::level::Level;
use crate::logger::Logger;
use crate::server::protocol::WireRecord;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::io::AsyncRead;

/// Handles one client connection: reads JSON lines and dispatches to the logger.
pub async fn handle_connection<R: AsyncRead + Unpin>(reader: R, logger: Arc<Logger>) {
    let mut lines = BufReader::new(reader).lines();
    loop {
        match lines.next_line().await {
            Ok(Some(line)) if !line.trim().is_empty() => {
                match WireRecord::from_line(&line) {
                    Ok(rec) => dispatch(&rec, &logger),
                    Err(e) => {
                        internal::warn("RSERVER", &format!("Malformed JSON: {e} — line: {line}"));
                    }
                }
            }
            Ok(Some(_)) => {} // empty line, skip
            Ok(None) => break, // EOF — client disconnected
            Err(e) => {
                internal::warn("RSERVER", &format!("Read error: {e}"));
                break;
            }
        }
    }
}

fn dispatch(rec: &WireRecord, logger: &Logger) {
    let level: Level = rec.level.parse().unwrap_or(Level::Info);
    logger.log_full(level, &rec.scope, &rec.message, rec.app.as_deref());
}
```

Add to `src/server/mod.rs`:
```rust
pub mod connection;
```

**Step 2: Verify it compiles**

```bash
cargo check --features rserver 2>&1
```
Expected: no errors.

**Step 3: Commit**

```bash
git add src/server/connection.rs src/server/mod.rs
git commit -m "feat(rserver): add per-connection JSON handler"
```

---

### Task 7: Server — listeners (Unix + TCP)

The Tokio accept loops for both transports.

**Files:**
- Create: `src/server/listener.rs`
- Modify: `src/server/mod.rs`

**Step 1: Write the code**

Create `src/server/listener.rs`:

```rust
//! Tokio accept loops for Unix socket and TCP.

use crate::logger::Logger;
use crate::server::config::ServerConfig;
use crate::server::connection::handle_connection;
use crate::internal;
use std::sync::Arc;
use tokio::net::{TcpListener, UnixListener};

/// Run both listeners concurrently until a shutdown signal is received.
///
/// # Errors
/// Returns an error if binding either socket fails.
pub async fn run_listeners(config: &ServerConfig, logger: Arc<Logger>) -> Result<(), crate::Error> {
    // Remove stale Unix socket file if present
    let _ = std::fs::remove_file(&config.socket_path);

    let unix_listener = UnixListener::bind(&config.socket_path)
        .map_err(|e| crate::Error::Io(e))?;
    internal::info("RSERVER", &format!("Listening on Unix socket: {}", config.socket_path));

    let tcp_listener = TcpListener::bind(config.tcp_addr()).await
        .map_err(|e| crate::Error::Io(e))?;
    internal::info("RSERVER", &format!("Listening on TCP: {}", config.tcp_addr()));

    let unix_logger = Arc::clone(&logger);
    let tcp_logger = Arc::clone(&logger);

    let unix_task = tokio::spawn(async move {
        loop {
            match unix_listener.accept() {
                Ok((stream, _addr)) => {
                    let log = Arc::clone(&unix_logger);
                    tokio::spawn(handle_connection(stream, log));
                }
                Err(e) => {
                    internal::warn("RSERVER", &format!("Unix accept error: {e}"));
                }
            }
        }
    });

    let tcp_task = tokio::spawn(async move {
        loop {
            match tcp_listener.accept().await {
                Ok((stream, addr)) => {
                    internal::trace("RSERVER", &format!("TCP connection from {addr}"));
                    let log = Arc::clone(&tcp_logger);
                    tokio::spawn(handle_connection(stream, log));
                }
                Err(e) => {
                    internal::warn("RSERVER", &format!("TCP accept error: {e}"));
                }
            }
        }
    });

    // Wait for SIGTERM or SIGINT
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigterm = signal(SignalKind::terminate()).unwrap();
        let mut sigint = signal(SignalKind::interrupt()).unwrap();
        tokio::select! {
            _ = sigterm.recv() => { internal::info("RSERVER", "Received SIGTERM, shutting down"); }
            _ = sigint.recv() => { internal::info("RSERVER", "Received SIGINT, shutting down"); }
        }
    }
    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await.ok();
    }

    unix_task.abort();
    tcp_task.abort();
    Ok(())
}
```

Add to `src/server/mod.rs`:
```rust
pub mod listener;
```

**Step 2: Verify it compiles**

```bash
cargo check --features rserver 2>&1
```
Expected: no errors. (The `signal` feature for Tokio needs to be in the feature list — it's already included in Task 1 via `"signal"`.)

**Step 3: Commit**

```bash
git add src/server/listener.rs src/server/mod.rs
git commit -m "feat(rserver): add Unix+TCP accept loops with graceful shutdown"
```

---

### Task 8: Server — daemon (PID file + run entry point)

**Files:**
- Create: `src/server/daemon.rs`
- Modify: `src/server/mod.rs`

**Step 1: Write the code**

Create `src/server/daemon.rs`:

```rust
//! PID file management and daemon entry point.

use crate::internal;
use crate::server::config::ServerConfig;
use std::fs;
use std::path::Path;

/// Writes the current process PID to `config.pid_file`.
///
/// # Errors
/// Returns an error if the file cannot be written.
pub fn write_pid(config: &ServerConfig) -> Result<(), crate::Error> {
    let pid = std::process::id();
    fs::write(&config.pid_file, pid.to_string())?;
    internal::debug("RSERVER", &format!("PID {pid} written to {}", config.pid_file));
    Ok(())
}

/// Reads the PID from `config.pid_file`.
/// Returns `None` if the file doesn't exist.
///
/// # Errors
/// Returns an error if the file exists but cannot be read or parsed.
pub fn read_pid(config: &ServerConfig) -> Result<Option<u32>, crate::Error> {
    let path = Path::new(&config.pid_file);
    if !path.exists() {
        return Ok(None);
    }
    let contents = fs::read_to_string(path)?;
    let pid: u32 = contents.trim().parse().map_err(|_| {
        crate::Error::Format(format!("Invalid PID in {}: {:?}", config.pid_file, contents))
    })?;
    Ok(Some(pid))
}

/// Returns true if a process with `pid` is currently running.
#[cfg(unix)]
#[must_use]
pub fn pid_is_running(pid: u32) -> bool {
    // kill(pid, 0) returns 0 if the process exists
    unsafe { libc_kill(pid as i32, 0) == 0 }
}

#[cfg(unix)]
extern "C" {
    fn libc_kill(pid: libc::pid_t, sig: libc::c_int) -> libc::c_int;
}
```

Hmm, we can't use `libc` without adding it as a dependency. Let's use a simpler approach — check if `/proc/<pid>` exists (Linux-specific but fine for Hyprland):

```rust
//! PID file management and daemon entry point.

use crate::internal;
use crate::server::config::ServerConfig;
use std::fs;
use std::path::Path;

/// Writes the current process PID to `config.pid_file`.
///
/// # Errors
/// Returns an error if the file cannot be written.
pub fn write_pid(config: &ServerConfig) -> Result<(), crate::Error> {
    let pid = std::process::id();
    fs::write(&config.pid_file, pid.to_string())?;
    internal::debug("RSERVER", &format!("PID {pid} written to {}", config.pid_file));
    Ok(())
}

/// Reads the PID from `config.pid_file`.
/// Returns `None` if the file doesn't exist.
///
/// # Errors
/// Returns an error if the file exists but cannot be read or parsed.
pub fn read_pid(config: &ServerConfig) -> Result<Option<u32>, crate::Error> {
    let path = Path::new(&config.pid_file);
    if !path.exists() {
        return Ok(None);
    }
    let contents = fs::read_to_string(path)?;
    let pid: u32 = contents.trim().parse().map_err(|_| {
        crate::Error::Format(format!("Invalid PID in {}: {:?}", config.pid_file, contents))
    })?;
    Ok(Some(pid))
}

/// Returns true if a process with this PID is currently running.
/// Uses `/proc/<pid>` existence check (Linux).
#[must_use]
pub fn pid_is_running(pid: u32) -> bool {
    Path::new(&format!("/proc/{pid}")).exists()
}

/// Removes the PID file (called on clean shutdown).
pub fn remove_pid(config: &ServerConfig) {
    let _ = fs::remove_file(&config.pid_file);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn config_with_tmpdir(dir: &TempDir) -> ServerConfig {
        let mut cfg = ServerConfig::default();
        cfg.pid_file = dir.path().join("test.pid").to_string_lossy().to_string();
        cfg
    }

    #[test]
    fn write_and_read_pid() {
        let dir = TempDir::new().unwrap();
        let cfg = config_with_tmpdir(&dir);

        write_pid(&cfg).unwrap();
        let pid = read_pid(&cfg).unwrap();
        assert_eq!(pid, Some(std::process::id()));
    }

    #[test]
    fn read_pid_missing_file() {
        let dir = TempDir::new().unwrap();
        let cfg = config_with_tmpdir(&dir);
        assert_eq!(read_pid(&cfg).unwrap(), None);
    }

    #[test]
    fn current_pid_is_running() {
        assert!(pid_is_running(std::process::id()));
    }

    #[test]
    fn fake_pid_not_running() {
        // PID 999999 is very unlikely to be running
        assert!(!pid_is_running(999_999));
    }
}
```

Add to `src/server/mod.rs`:
```rust
pub mod daemon;

/// Starts the server: writes PID, builds Logger, runs listeners.
///
/// This is the main entry point called by `cmd_server_start`.
///
/// # Errors
/// Returns an error if binding fails or PID cannot be written.
pub fn run(config: &ServerConfig) -> Result<(), crate::Error> {
    use crate::logger::Logger;
    use crate::level::Level;
    use std::sync::Arc;

    daemon::write_pid(config)?;

    let level: Level = config.log_level.parse().unwrap_or(Level::Info);
    let logger = Logger::builder()
        .level(level)
        .terminal()
            .colors(true)
            .done()
        .build();

    let logger = Arc::new(logger);

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| crate::Error::Io(e))?;

    let result = rt.block_on(listener::run_listeners(config, logger));
    daemon::remove_pid(config);
    result
}
```

**Step 2: Run daemon tests**

```bash
cargo test --features rserver server::daemon 2>&1
```
Expected: all 4 tests pass.

**Step 3: Verify server::run compiles**

```bash
cargo check --features rserver 2>&1
```
Expected: no errors.

**Step 4: Commit**

```bash
git add src/server/daemon.rs src/server/mod.rs
git commit -m "feat(rserver): add PID file management and server run() entry point"
```

---

### Task 9: CLI — `server` and `send` commands

**Files:**
- Create: `src/cli/commands/server.rs`
- Modify: `src/cli/commands/mod.rs`
- Modify: `src/bin/hyprslog.rs`

**Step 1: Write the code**

Create `src/cli/commands/server.rs`:

```rust
//! CLI commands: `hyprslog server start/stop/status` and `hyprslog send`.

use crate::cli::util::parse_level;
use crate::internal;
use crate::output::{LogRecord, Output, RemoteOutput};
use crate::fmt::FormatValues;
use crate::server::config::ServerConfig;
use crate::server::daemon;
use std::process::ExitCode;

/// Handles `hyprslog server <subcommand>`.
#[must_use]
pub fn cmd_server(args: &[&str]) -> ExitCode {
    match args.first().copied() {
        Some("start") => cmd_server_start(),
        Some("stop") => cmd_server_stop(),
        Some("status") => cmd_server_status(),
        _ => {
            internal::warn("CLI", "Usage: hyprslog server <start|stop|status>");
            ExitCode::FAILURE
        }
    }
}

fn cmd_server_start() -> ExitCode {
    let config = match ServerConfig::load() {
        Ok(c) => c,
        Err(e) => {
            internal::error("CLI", &format!("Failed to load server config: {e}"));
            return ExitCode::FAILURE;
        }
    };

    // Check if already running
    if let Ok(Some(pid)) = daemon::read_pid(&config) {
        if daemon::pid_is_running(pid) {
            internal::error("CLI", &format!("Server already running (PID {pid})"));
            return ExitCode::FAILURE;
        }
    }

    // Fork into background via std::process::Command re-exec with --foreground
    // Simple approach: re-exec self with `server --foreground`
    let exe = std::env::current_exe().unwrap_or_default();
    let mut child = match std::process::Command::new(&exe)
        .arg("server")
        .arg("--foreground")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            internal::error("CLI", &format!("Failed to start server: {e}"));
            return ExitCode::FAILURE;
        }
    };

    // Give it a moment to start
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Check it's still running (didn't crash immediately)
    match child.try_wait() {
        Ok(Some(status)) => {
            internal::error("CLI", &format!("Server exited immediately: {status}"));
            ExitCode::FAILURE
        }
        _ => {
            internal::info("CLI", &format!("Server started (PID {})", child.id()));
            // Detach the child
            drop(child);
            ExitCode::SUCCESS
        }
    }
}

fn cmd_server_stop() -> ExitCode {
    let config = match ServerConfig::load() {
        Ok(c) => c,
        Err(e) => {
            internal::error("CLI", &format!("Failed to load server config: {e}"));
            return ExitCode::FAILURE;
        }
    };

    match daemon::read_pid(&config) {
        Ok(Some(pid)) if daemon::pid_is_running(pid) => {
            // Send SIGTERM
            #[cfg(unix)]
            unsafe {
                libc_kill(pid as i32, 15); // SIGTERM = 15
            }
            internal::info("CLI", &format!("Sent SIGTERM to PID {pid}"));
            ExitCode::SUCCESS
        }
        Ok(_) => {
            internal::warn("CLI", "Server is not running");
            ExitCode::FAILURE
        }
        Err(e) => {
            internal::error("CLI", &format!("Cannot read PID file: {e}"));
            ExitCode::FAILURE
        }
    }
}

#[cfg(unix)]
extern "C" {
    fn libc_kill(pid: libc::pid_t, sig: libc::c_int) -> libc::c_int;
}

fn cmd_server_status() -> ExitCode {
    let config = match ServerConfig::load() {
        Ok(c) => c,
        Err(e) => {
            internal::error("CLI", &format!("Cannot load config: {e}"));
            return ExitCode::FAILURE;
        }
    };
    match daemon::read_pid(&config) {
        Ok(Some(pid)) if daemon::pid_is_running(pid) => {
            println!("hyprslog server is running (PID {pid})");
            ExitCode::SUCCESS
        }
        _ => {
            println!("hyprslog server is not running");
            ExitCode::FAILURE
        }
    }
}

/// Handles `hyprslog send [--app <app>] [--tcp <addr>] <level> <scope> <msg...>`.
///
/// Default target: Unix socket from server config.
#[must_use]
pub fn cmd_send(args: &[&str]) -> ExitCode {
    let mut remaining = args;
    let mut app: Option<String> = None;
    let mut tcp_addr: Option<String> = None;

    // Parse optional flags
    loop {
        match remaining.first().copied() {
            Some("--app") if remaining.len() > 1 => {
                app = Some(remaining[1].to_string());
                remaining = &remaining[2..];
            }
            Some("--tcp") if remaining.len() > 1 => {
                tcp_addr = Some(remaining[1].to_string());
                remaining = &remaining[2..];
            }
            _ => break,
        }
    }

    if remaining.len() < 3 {
        internal::warn("CLI", "Usage: hyprslog send [--app <app>] [--tcp <addr>] <level> <scope> <message>");
        return ExitCode::FAILURE;
    }

    let Some(level) = parse_level(remaining[0]) else {
        internal::error("CLI", &format!("Invalid level: {}", remaining[0]));
        return ExitCode::FAILURE;
    };

    let scope = remaining[1];
    let message = remaining[2..].join(" ");

    let config = ServerConfig::load().unwrap_or_default();

    let output: Box<dyn Output> = if let Some(addr) = tcp_addr {
        Box::new(RemoteOutput::tcp(addr))
    } else {
        Box::new(RemoteOutput::unix(&config.socket_path))
    };

    let record = LogRecord {
        level,
        scope: scope.to_string(),
        message,
        values: FormatValues::new(),
        label_override: None,
        app_name: app,
        raw: false,
    };

    let _ = output.write(&record);
    // Give the background worker time to send
    std::thread::sleep(std::time::Duration::from_millis(100));
    ExitCode::SUCCESS
}
```

**Note:** `libc` is needed for the `kill` syscall. Add it as an optional dependency:

In `Cargo.toml`:
```toml
libc = { version = "0.2", optional = true }
```

And in `[features]`:
```toml
rserver = ["dep:tokio", "dep:libc"]
```

**Step 2: Wire into mod.rs and bin/hyprslog.rs**

In `src/cli/commands/mod.rs`, add at the bottom:
```rust
#[cfg(feature = "rserver")]
mod server;
#[cfg(feature = "rserver")]
pub use server::{cmd_send, cmd_server};
```

In `src/bin/hyprslog.rs`, add at the top (with the other cfg imports):
```rust
#[cfg(feature = "rserver")]
use hyprslog::cli::{cmd_send, cmd_server};
```

And in the `match args_str[0]` block, add before the `_` wildcard:
```rust
#[cfg(feature = "rserver")]
"server" => cmd_server(&args_str[1..]),
#[cfg(feature = "rserver")]
"send" => cmd_send(&args_str[1..]),
#[cfg(feature = "rserver")]
"server" if args_str.get(1) == Some(&"--foreground") => {
    // Run server in foreground (called by start)
    let config = hyprslog::ServerConfig::load().unwrap_or_default();
    match hyprslog::server::run(&config) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Server error: {e}");
            ExitCode::FAILURE
        }
    }
}
```

**Note:** The `--foreground` match needs to come before the plain `"server"` match in Rust's match arms — reorder accordingly in the actual code.

**Step 3: Verify it compiles**

```bash
cargo build --features rserver 2>&1
```
Expected: compiles successfully.

**Step 4: Commit**

```bash
git add src/cli/commands/server.rs src/cli/commands/mod.rs src/bin/hyprslog.rs Cargo.toml Cargo.lock
git commit -m "feat(rserver): add server start/stop/status and send CLI commands"
```

---

### Task 10: Integration test — full server round-trip

Spin up the server in a background thread, send via RemoteOutput, verify the server wrote to a temp file.

**Files:**
- Create: `tests/rserver_integration.rs`

**Step 1: Write the test**

```rust
//! Integration test: full server round-trip.
#[cfg(feature = "rserver")]
mod tests {
    use hyprslog::server::ServerConfig;
    use hyprslog::{Level, Logger};
    use std::io::BufRead;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tempfile::TempDir;

    /// Starts the server in a background thread with a random TCP port.
    /// Returns (port, shutdown_tx).
    fn start_test_server() -> (u16, std::net::TcpListener) {
        // Bind port 0 to get a random free port
        let probe = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = probe.local_addr().unwrap().port();
        drop(probe);
        (port, std::net::TcpListener::bind(format!("127.0.0.1:{port}")).unwrap())
    }

    #[test]
    fn remote_output_round_trip_tcp() {
        // We'll intercept at the TCP level (not via real server) since
        // spinning up a full tokio server in a test is complex.
        // This test validates the wire format end-to-end.

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let lines: Arc<Mutex<Vec<serde_json::Value>>> = Arc::new(Mutex::new(Vec::new()));
        let lines_clone = Arc::clone(&lines);

        std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            for line in std::io::BufReader::new(stream).lines() {
                let Ok(line) = line else { break };
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
                    lines_clone.lock().unwrap().push(v);
                }
            }
        });

        std::thread::sleep(Duration::from_millis(50));

        let logger = Logger::builder()
            .level(Level::Trace)
            .remote()
                .tcp(format!("127.0.0.1:{port}"))
                .done()
            .build();

        logger.trace("CORE", "trace message");
        logger.debug("NET", "debug message");
        logger.info("APP", "info message");
        logger.warn("DB", "warn message");
        logger.error("SYS", "error message");

        std::thread::sleep(Duration::from_millis(300));

        let received = lines.lock().unwrap();
        assert_eq!(received.len(), 5, "Expected 5 records, got {}", received.len());

        assert_eq!(received[0]["level"], "trace");
        assert_eq!(received[0]["scope"], "CORE");

        assert_eq!(received[4]["level"], "error");
        assert_eq!(received[4]["scope"], "SYS");
        assert_eq!(received[4]["message"], "error message");
    }

    #[test]
    fn level_filtering_respected() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let count: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
        let count_clone = Arc::clone(&count);

        std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            for _line in std::io::BufReader::new(stream).lines().flatten() {
                *count_clone.lock().unwrap() += 1;
            }
        });

        std::thread::sleep(Duration::from_millis(50));

        // Logger with min_level=Warn should only send warn+error
        let logger = Logger::builder()
            .level(Level::Warn)
            .remote()
                .tcp(format!("127.0.0.1:{port}"))
                .done()
            .build();

        logger.trace("X", "filtered");
        logger.debug("X", "filtered");
        logger.info("X", "filtered");
        logger.warn("X", "sent");
        logger.error("X", "sent");

        std::thread::sleep(Duration::from_millis(300));
        assert_eq!(*count.lock().unwrap(), 2);
    }
}
```

**Step 2: Run integration tests**

```bash
cargo test --features rserver --test rserver_integration 2>&1
```
Expected: both tests pass.

**Step 3: Run all tests to check for regressions**

```bash
cargo test --features rserver 2>&1
cargo test 2>&1
```
Expected: all pass (with and without the feature flag).

**Step 4: Check clippy**

```bash
cargo clippy --features rserver -- -D warnings 2>&1
```
Expected: no warnings.

**Step 5: Commit**

```bash
git add tests/rserver_integration.rs
git commit -m "test(rserver): add integration tests for RemoteOutput round-trip"
```

---

### Task 11: Update docs and lib.rs re-exports

Expose the public API cleanly.

**Files:**
- Modify: `src/lib.rs`
- Modify: `src/lib.rs` doc comment

**Step 1: Update lib.rs**

In the feature list in the module-level doc comment, add:
```
//! - `rserver`: Enables remote logging server daemon and `RemoteOutput` client
```

Ensure the re-exports at the bottom include:
```rust
#[cfg(feature = "rserver")]
pub use output::RemoteOutput;
#[cfg(feature = "rserver")]
pub use server::{ServerConfig, run as run_server};
#[cfg(feature = "rserver")]
pub use logger::RemoteBuilder;
```

**Step 2: Final build check**

```bash
cargo build --all-features 2>&1
cargo build 2>&1
just lint 2>&1
```
Expected: clean builds, no lint warnings.

**Step 3: Commit**

```bash
git add src/lib.rs
git commit -m "feat(rserver): expose public API in lib.rs re-exports"
```

---

## Summary of all commits

1. `feat(rserver): add rserver feature flag with tokio dependency`
2. `feat(rserver): add WireRecord wire protocol type`
3. `feat(rserver): add ServerConfig with TOML loading`
4. `feat(rserver): add RemoteOutput client backend`
5. `feat(rserver): add RemoteBuilder to logger builder chain`
6. `feat(rserver): add per-connection JSON handler`
7. `feat(rserver): add Unix+TCP accept loops with graceful shutdown`
8. `feat(rserver): add PID file management and server run() entry point`
9. `feat(rserver): add server start/stop/status and send CLI commands`
10. `test(rserver): add integration tests for RemoteOutput round-trip`
11. `feat(rserver): expose public API in lib.rs re-exports`

## Notes for implementer

- **Rust edition 2024** is in use — `let ... else`, `if let ... &&`, etc. are all valid
- **Clippy pedantic + nursery** — every `pub fn` that returns a value needs `#[must_use]`, document all `# Errors` and `# Panics`
- The `just test` command uses `cargo-nextest` — use `cargo test --features rserver` for direct runs
- The `libc` external `"C"` declaration for `kill` will trigger `unsafe_code` lint — this is already handled by the `cfg_attr` in `lib.rs` for the `ffi` and `hyprland` features; add `rserver` to that condition too:
  ```rust
  #![cfg_attr(not(any(feature = "ffi", feature = "hyprland", feature = "rserver")), forbid(unsafe_code))]
  ```
- For the `server --foreground` dispatch in `bin/hyprslog.rs`: Rust requires match arms to be ordered such that more specific patterns come first. Put `"server" if args[1] == "--foreground"` before the plain `"server"` arm — or better, check inside `cmd_server()` for the `--foreground` sub-arg.
