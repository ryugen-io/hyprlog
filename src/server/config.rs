//! Server daemon configuration.

use serde::Deserialize;
use std::path::PathBuf;

fn default_socket_path() -> String {
    std::env::var("XDG_RUNTIME_DIR").map_or_else(
        |_| "/tmp/hyprlog.sock".to_string(),
        |runtime| format!("{runtime}/hyprlog.sock"),
    )
}

fn default_pid_file() -> String {
    std::env::var("XDG_RUNTIME_DIR").map_or_else(
        |_| "/tmp/hyprlog.pid".to_string(),
        |runtime| format!("{runtime}/hyprlog.pid"),
    )
}

fn default_tcp_bind() -> String {
    "127.0.0.1".to_string()
}

const fn default_tcp_port() -> u16 {
    9872
}

fn default_log_level() -> String {
    "info".to_string()
}

const fn default_terminal_colors() -> bool {
    true
}

const fn default_terminal_enabled() -> bool {
    true
}

/// Configuration for the hyprlog server daemon.
///
/// Loaded from `~/.config/hypr/hyprlog-server.toml`.
/// All fields have sensible defaults — the file is optional.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    /// Path to the Unix domain socket.
    #[serde(default = "default_socket_path")]
    pub socket_path: String,
    /// TCP port (0 = TCP disabled).
    #[serde(default = "default_tcp_port")]
    pub tcp_port: u16,
    /// TCP bind address.
    #[serde(default = "default_tcp_bind")]
    pub tcp_bind: String,
    /// Path to the PID file.
    #[serde(default = "default_pid_file")]
    pub pid_file: String,
    /// Minimum log level for the server's outputs.
    #[serde(default = "default_log_level")]
    pub log_level: String,
    /// Enable terminal output on the server.
    #[serde(default = "default_terminal_enabled")]
    pub terminal_enabled: bool,
    /// Enable ANSI colors on terminal output.
    #[serde(default = "default_terminal_colors")]
    pub terminal_colors: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            socket_path: default_socket_path(),
            tcp_port: default_tcp_port(),
            tcp_bind: default_tcp_bind(),
            pid_file: default_pid_file(),
            log_level: default_log_level(),
            terminal_enabled: default_terminal_enabled(),
            terminal_colors: default_terminal_colors(),
        }
    }
}

impl ServerConfig {
    /// Returns the path to the server config file (`~/.config/hypr/server.conf`).
    #[must_use]
    pub fn config_path() -> PathBuf {
        directories::BaseDirs::new().map_or_else(
            || PathBuf::from("server.conf"),
            |dirs| {
                dirs.config_dir()
                    .join("hypr")
                    .join("server.conf")
            },
        )
    }

    /// Loads config from `~/.config/hypr/hyprlog-server.toml`.
    ///
    /// Returns defaults silently if the file does not exist.
    ///
    /// # Errors
    /// Returns an error if the file exists but cannot be read or parsed.
    pub fn load() -> Result<Self, crate::Error> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = std::fs::read_to_string(&path)?;
        let config: Self = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Returns the full TCP bind address string (`host:port`).
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
        assert!(!cfg.tcp_bind.is_empty());
        assert!(!cfg.pid_file.is_empty());
        assert!(!cfg.log_level.is_empty());
    }

    #[test]
    fn tcp_addr_format() {
        let cfg = ServerConfig {
            tcp_bind: "0.0.0.0".to_string(),
            tcp_port: 1234,
            ..ServerConfig::default()
        };
        assert_eq!(cfg.tcp_addr(), "0.0.0.0:1234");
    }

    #[test]
    fn load_returns_defaults_when_missing() {
        // Very unlikely the test runner has a hyprlog-server.toml in its home.
        // If it does, we just verify load() doesn't error.
        assert!(ServerConfig::load().is_ok());
    }

    #[test]
    fn toml_deserialize() {
        let toml = r#"
socket_path = "/tmp/test.sock"
tcp_port = 1234
tcp_bind = "0.0.0.0"
log_level = "debug"
"#;
        let cfg: ServerConfig = toml::from_str(toml).unwrap();
        assert_eq!(cfg.socket_path, "/tmp/test.sock");
        assert_eq!(cfg.tcp_port, 1234);
        assert_eq!(cfg.tcp_bind, "0.0.0.0");
        assert_eq!(cfg.log_level, "debug");
    }
}
