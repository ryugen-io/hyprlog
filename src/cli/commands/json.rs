//! Programs that already emit structured JSON shouldn't need to decompose it into
//! positional args — piping JSONL directly into hyprlog is faster and less error-prone.

use crate::cli::util::parse_level;
use crate::internal;
use crate::logger::Logger;
use serde::Deserialize;
use std::io::{self, BufRead};
use std::process::ExitCode;

/// Minimal required fields — anything beyond level/scope/msg would break compatibility with simple JSON producers.
#[derive(Debug, Deserialize)]
struct JsonLogEntry {
    level: String,
    scope: String,
    msg: String,
}

/// Accepts a single JSON string or reads JSONL from stdin — supports both one-shot and piped workflows.
#[must_use]
pub fn cmd_json(input: Option<&str>, logger: &Logger) -> ExitCode {
    let mut processed = 0u64;
    let mut failed = 0u64;

    let process_line = |line: &str| -> Result<(), String> {
        let entry: JsonLogEntry =
            serde_json::from_str(line).map_err(|e| format!("invalid JSON: {e}"))?;

        let level =
            parse_level(&entry.level).ok_or_else(|| format!("invalid level: {}", entry.level))?;

        logger.log(level, &entry.scope, &entry.msg);
        Ok(())
    };

    match input {
        None | Some("-") => {
            internal::debug("JSON", "Reading JSON from stdin");
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                match line {
                    Ok(l) if !l.trim().is_empty() => {
                        internal::trace("JSON", "Processing JSON line");
                        if let Err(e) = process_line(&l) {
                            internal::error("JSON", &e);
                            failed += 1;
                        } else {
                            processed += 1;
                        }
                    }
                    Ok(_) => {}
                    Err(e) => {
                        internal::error("JSON", &format!("Error reading stdin: {e}"));
                        return ExitCode::FAILURE;
                    }
                }
            }
            internal::info(
                "JSON",
                &format!("JSON: processed {processed} entries, {failed} failed"),
            );
            ExitCode::SUCCESS
        }
        Some(json) => {
            internal::trace("JSON", "Processing JSON line");
            match process_line(json) {
                Ok(()) => {
                    internal::info("JSON", "JSON: processed 1 entry, 0 failed");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    internal::error("JSON", &e);
                    internal::info("JSON", "JSON: processed 0 entries, 1 failed");
                    ExitCode::FAILURE
                }
            }
        }
    }
}
