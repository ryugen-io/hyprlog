//! Wire protocol for the rserver.
//!
//! One JSON line per log record, newline-terminated. Fire-and-forget.

use crate::level::Level;
use serde::{Deserialize, Serialize};

/// A log record as transmitted over the wire.
///
/// Serializes to a single newline-terminated JSON line:
/// ```json
/// {"level":"info","scope":"NET","message":"Connected"}
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireRecord {
    /// Log level as a lowercase string.
    pub level: String,
    /// Scope / component name.
    pub scope: String,
    /// Optional application name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<String>,
    /// Log message.
    pub message: String,
}

impl WireRecord {
    /// Constructs a wire record from its parts.
    #[must_use]
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
    /// Returns an error if serialization fails (this is effectively infallible for this type).
    pub fn to_line(&self) -> Result<String, serde_json::Error> {
        let mut s = serde_json::to_string(self)?;
        s.push('\n');
        Ok(s)
    }

    /// Parses a newline-terminated (or plain) JSON string into a [`WireRecord`].
    ///
    /// # Errors
    /// Returns an error if `line` is not valid JSON matching the wire schema.
    pub fn from_line(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line.trim())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(!line.contains("\"app\""), "app field must be absent when None");
        let parsed = WireRecord::from_line(&line).unwrap();
        assert_eq!(parsed, rec);
    }

    #[test]
    fn level_string_values() {
        for (level, expected) in [
            (Level::Trace, "trace"),
            (Level::Debug, "debug"),
            (Level::Info, "info"),
            (Level::Warn, "warn"),
            (Level::Error, "error"),
        ] {
            let rec = WireRecord::from_parts(level, "X", None, "m");
            assert!(rec.to_line().unwrap().contains(expected));
        }
    }

    #[test]
    fn from_line_trims_whitespace() {
        let line = "  {\"level\":\"error\",\"scope\":\"S\",\"message\":\"m\"}  \n";
        let rec = WireRecord::from_line(line).unwrap();
        assert_eq!(rec.level, "error");
    }
}
