//! Remote output: forwards log records to a hyprslog server over the network.

use crate::internal;
use crate::output::{LogRecord, Output};
use crate::server::protocol::WireRecord;
use std::net::TcpStream;
use std::os::unix::net::UnixStream;
use std::sync::mpsc::{self, SyncSender};
use std::thread;

// ── Transport ────────────────────────────────────────────────────────────────

/// Connection target for a [`RemoteOutput`].
#[derive(Debug, Clone)]
pub enum RemoteTarget {
    /// Unix domain socket path.
    Unix(String),
    /// TCP address in `host:port` form.
    Tcp(String),
}

/// An active blocking connection to the server.
enum Conn {
    Unix(UnixStream),
    Tcp(TcpStream),
}

use std::io::Write;

impl Write for Conn {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Unix(s) => s.write(buf),
            Self::Tcp(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Unix(s) => s.flush(),
            Self::Tcp(s) => s.flush(),
        }
    }
}

impl RemoteTarget {
    fn connect(&self) -> std::io::Result<Conn> {
        match self {
            Self::Unix(path) => UnixStream::connect(path).map(Conn::Unix),
            Self::Tcp(addr) => TcpStream::connect(addr).map(Conn::Tcp),
        }
    }
}

// ── Worker thread ─────────────────────────────────────────────────────────────

/// Background thread: receives [`WireRecord`]s, sends them to the server.
/// Reconnects transparently if the connection drops.
#[allow(clippy::needless_pass_by_value)] // target must be owned (moved into thread)
fn worker(target: RemoteTarget, rx: mpsc::Receiver<WireRecord>) {
    let mut conn: Option<Conn> = None;

    for record in rx {
        let Ok(line) = record.to_line() else {
            continue;
        };
        let bytes = line.as_bytes();

        if !send_with_retry(&target, &mut conn, bytes) {
            internal::trace("REMOTE", "dropped record: server unreachable");
        }
    }
}

/// Tries to write `bytes` to the current connection, reconnecting once on failure.
/// Returns `true` if the bytes were successfully written.
fn send_with_retry(target: &RemoteTarget, conn: &mut Option<Conn>, bytes: &[u8]) -> bool {
    // target is borrowed here; worker owns it but only reads via reference.
    // First attempt with the existing connection.
    if let Some(c) = conn.as_mut() {
        if c.write_all(bytes).is_ok() {
            return true;
        }
        // Connection broken: drop it and reconnect.
        *conn = None;
    }

    // Reconnect and retry once.
    *conn = target.connect().ok();
    if let Some(c) = conn.as_mut() {
        if c.write_all(bytes).is_ok() {
            return true;
        }
        *conn = None; // failed again — leave disconnected
    }

    false
}

// ── RemoteOutput ──────────────────────────────────────────────────────────────

/// Output backend that forwards log records to a running hyprslog server.
///
/// `write()` is non-blocking: records are enqueued in a bounded channel and
/// sent by a dedicated background OS thread. Records are silently dropped when
/// the channel is full (capacity: 1 024) or when the server is unreachable.
pub struct RemoteOutput {
    sender: SyncSender<WireRecord>,
    // Keeps the background thread alive.
    _worker: thread::JoinHandle<()>,
}

impl RemoteOutput {
    /// Creates a [`RemoteOutput`] that connects via a Unix domain socket.
    #[must_use]
    pub fn unix(path: impl Into<String>) -> Self {
        Self::new(RemoteTarget::Unix(path.into()))
    }

    /// Creates a [`RemoteOutput`] that connects via TCP (`host:port`).
    #[must_use]
    pub fn tcp(addr: impl Into<String>) -> Self {
        Self::new(RemoteTarget::Tcp(addr.into()))
    }

    fn new(target: RemoteTarget) -> Self {
        let (tx, rx) = mpsc::sync_channel(1024);
        let handle = thread::spawn(move || worker(target, rx));
        Self {
            sender: tx,
            _worker: handle,
        }
    }
}

impl Output for RemoteOutput {
    fn write(&self, record: &LogRecord) -> Result<(), crate::Error> {
        // Construct wire record; skip raw log lines (no structured data).
        if record.raw {
            return Ok(());
        }
        let wire = WireRecord::from_parts(
            record.level,
            &record.scope,
            record.app_name.as_deref(),
            &record.message,
        );
        // try_send: never blocks. Full/disconnected → silently drop.
        let _ = self.sender.try_send(wire);
        Ok(())
    }

    fn flush(&self) -> Result<(), crate::Error> {
        // Fire-and-forget: no flush guarantee across the network.
        Ok(())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fmt::FormatValues;
    use crate::level::Level;
    use std::io::BufRead as _;
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
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let received: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let recv_clone = Arc::clone(&received);

        thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            for line in std::io::BufReader::new(stream).lines().flatten() {
                recv_clone.lock().unwrap().push(line);
            }
        });

        thread::sleep(Duration::from_millis(30));

        let output = RemoteOutput::tcp(addr.to_string());
        output
            .write(&make_record(Level::Info, "TEST", "hello world"))
            .unwrap();

        thread::sleep(Duration::from_millis(200));

        let lines = received.lock().unwrap();
        assert_eq!(lines.len(), 1);
        let v: serde_json::Value = serde_json::from_str(&lines[0]).unwrap();
        assert_eq!(v["level"], "info");
        assert_eq!(v["scope"], "TEST");
        assert_eq!(v["message"], "hello world");
    }

    #[test]
    fn sends_multiple_records_in_order() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let received: Arc<Mutex<Vec<serde_json::Value>>> = Arc::new(Mutex::new(Vec::new()));
        let recv_clone = Arc::clone(&received);

        thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            for line in std::io::BufReader::new(stream).lines().flatten() {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
                    recv_clone.lock().unwrap().push(v);
                }
            }
        });

        thread::sleep(Duration::from_millis(30));
        let output = RemoteOutput::tcp(addr.to_string());

        for i in 0..5u8 {
            output
                .write(&make_record(Level::Debug, "SEQ", &format!("msg {i}")))
                .unwrap();
        }

        thread::sleep(Duration::from_millis(300));
        let vals = received.lock().unwrap();
        assert_eq!(vals.len(), 5);
        for (i, v) in vals.iter().enumerate() {
            assert_eq!(v["message"], format!("msg {i}"));
        }
    }

    #[test]
    fn drops_gracefully_when_server_absent() {
        // Nothing listening on this port.
        let output = RemoteOutput::tcp("127.0.0.1:19999");
        // Must not panic or block.
        assert!(output
            .write(&make_record(Level::Warn, "X", "dropped"))
            .is_ok());
    }

    #[test]
    fn skips_raw_records() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let received: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let recv_clone = Arc::clone(&received);

        thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            for line in std::io::BufReader::new(stream).lines().flatten() {
                recv_clone.lock().unwrap().push(line);
            }
        });

        thread::sleep(Duration::from_millis(30));

        let output = RemoteOutput::tcp(addr.to_string());
        let mut raw_record = make_record(Level::Info, "", "raw text");
        raw_record.raw = true;
        output.write(&raw_record).unwrap();

        // Send a real record after so the connection is established
        output
            .write(&make_record(Level::Info, "Y", "normal"))
            .unwrap();

        thread::sleep(Duration::from_millis(200));

        let lines = received.lock().unwrap();
        // Only the normal record should arrive
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("normal"));
    }
}
