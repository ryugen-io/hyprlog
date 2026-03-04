//! Builder for the remote output backend.

use crate::logger::builder::LoggerBuilder;
use crate::output::RemoteOutput;

enum Target {
    Unix(String),
    Tcp(String),
}

/// Builder for [`RemoteOutput`] configuration.
///
/// Obtained via [`LoggerBuilder::remote`].
pub struct RemoteBuilder {
    pub(crate) parent: LoggerBuilder,
    target: Option<Target>,
}

impl RemoteBuilder {
    pub(crate) const fn new(parent: LoggerBuilder) -> Self {
        Self {
            parent,
            target: None,
        }
    }

    /// Connect via a Unix domain socket at `path`.
    #[must_use]
    pub fn socket(mut self, path: impl Into<String>) -> Self {
        self.target = Some(Target::Unix(path.into()));
        self
    }

    /// Connect via TCP to `addr` (e.g. `"127.0.0.1:9872"`).
    #[must_use]
    pub fn tcp(mut self, addr: impl Into<String>) -> Self {
        self.target = Some(Target::Tcp(addr.into()));
        self
    }

    /// Finishes remote configuration and returns to the [`LoggerBuilder`].
    ///
    /// # Panics
    /// Panics if neither [`socket`](Self::socket) nor [`tcp`](Self::tcp)
    /// was called before `done`.
    #[must_use]
    pub fn done(mut self) -> LoggerBuilder {
        let output = match self
            .target
            .take()
            .expect("call .socket() or .tcp() before .done()")
        {
            Target::Unix(path) => RemoteOutput::unix(path),
            Target::Tcp(addr) => RemoteOutput::tcp(addr),
        };
        self.parent.outputs.push(Box::new(output));
        self.parent
    }
}
