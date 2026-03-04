//! Per-connection async handler for the rserver.

use crate::internal;
use crate::logger::Logger;
use crate::server::protocol::WireRecord;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};

/// Handles one client connection by reading newline-delimited JSON records
/// and dispatching each through the shared [`Logger`].
///
/// Exits when the client disconnects (EOF) or a read error occurs.
pub async fn handle_connection<R>(reader: R, logger: Arc<Logger>)
where
    R: AsyncRead + Unpin,
{
    let mut lines = BufReader::new(reader).lines();
    loop {
        match lines.next_line().await {
            Ok(Some(line)) if !line.trim().is_empty() => {
                match WireRecord::from_line(&line) {
                    Ok(rec) => dispatch(&rec, &logger),
                    Err(e) => {
                        internal::warn(
                            "RSERVER",
                            &format!("malformed JSON ({e}): {line}"),
                        );
                    }
                }
            }
            Ok(Some(_)) => {} // blank line — skip
            Ok(None) => break, // EOF
            Err(e) => {
                internal::trace("RSERVER", &format!("read error: {e}"));
                break;
            }
        }
    }
}

fn dispatch(rec: &WireRecord, logger: &Logger) {
    let level = rec.level.parse().unwrap_or(crate::level::Level::Info);
    logger.log_full(level, &rec.scope, &rec.message, rec.app.as_deref());
}
