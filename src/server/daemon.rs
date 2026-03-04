//! PID-file management for the server daemon.

use crate::internal;
use crate::server::config::ServerConfig;
use std::fs;
use std::path::Path;

/// Writes the current process PID to the file named in `config.pid_file`.
///
/// # Errors
/// Returns an error if the file cannot be created or written.
pub fn write_pid(config: &ServerConfig) -> Result<(), crate::Error> {
    let pid = std::process::id();
    fs::write(&config.pid_file, pid.to_string())?;
    internal::debug("RSERVER", &format!("PID {pid} → {}", config.pid_file));
    Ok(())
}

/// Reads the PID from the file named in `config.pid_file`.
///
/// Returns `None` if the file does not exist.
///
/// # Errors
/// Returns an error if the file exists but cannot be read or contains a non-numeric value.
pub fn read_pid(config: &ServerConfig) -> Result<Option<u32>, crate::Error> {
    let path = Path::new(&config.pid_file);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path)?;
    raw.trim()
        .parse::<u32>()
        .map(Some)
        .map_err(|_| crate::Error::Format(format!("invalid PID in {}: {:?}", config.pid_file, raw.trim())))
}

/// Returns `true` if a process with the given PID is currently running.
///
/// Uses `/proc/<pid>` presence (Linux). Always returns `false` on non-Linux.
#[must_use]
pub fn pid_is_running(pid: u32) -> bool {
    Path::new(&format!("/proc/{pid}")).exists()
}

/// Removes the PID file. Errors are silently ignored.
pub fn remove_pid(config: &ServerConfig) {
    let _ = fs::remove_file(&config.pid_file);
}

/// Sends `SIGTERM` to the given PID via the system `kill` command.
///
/// # Errors
/// Returns an error if the `kill` command cannot be spawned.
pub fn send_sigterm(pid: u32) -> Result<(), crate::Error> {
    std::process::Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .status()
        .map(|_| ())
        .map_err(crate::Error::Io)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn cfg(dir: &TempDir) -> ServerConfig {
        ServerConfig {
            pid_file: dir.path().join("test.pid").to_string_lossy().into_owned(),
            ..ServerConfig::default()
        }
    }

    #[test]
    fn write_and_read_roundtrip() {
        let dir = TempDir::new().unwrap();
        let config = cfg(&dir);
        write_pid(&config).unwrap();
        assert_eq!(read_pid(&config).unwrap(), Some(std::process::id()));
    }

    #[test]
    fn read_returns_none_when_missing() {
        let dir = TempDir::new().unwrap();
        let config = cfg(&dir);
        assert_eq!(read_pid(&config).unwrap(), None);
    }

    #[test]
    fn current_process_is_running() {
        assert!(pid_is_running(std::process::id()));
    }

    #[test]
    fn nonexistent_pid_not_running() {
        assert!(!pid_is_running(999_999));
    }

    #[test]
    fn remove_pid_is_idempotent() {
        let dir = TempDir::new().unwrap();
        let config = cfg(&dir);
        remove_pid(&config); // no file yet — should not panic
        write_pid(&config).unwrap();
        remove_pid(&config);
        assert!(!Path::new(&config.pid_file).exists());
    }
}
