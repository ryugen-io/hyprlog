//! Error types for cleanup operations.

/// Error type for cleanup operations.
#[derive(Debug)]
pub enum CleanupError {
    /// I/O error.
    Io(std::io::Error),
    /// Invalid path.
    InvalidPath(String),
}

impl std::fmt::Display for CleanupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::InvalidPath(s) => write!(f, "invalid path: {s}"),
        }
    }
}

impl std::error::Error for CleanupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::InvalidPath(_) => None,
        }
    }
}

impl From<std::io::Error> for CleanupError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
