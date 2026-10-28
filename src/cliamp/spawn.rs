use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

use super::paths::socket_path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionPhase {
    NoBinary,
    Connecting,
    Connected,
    Failed(String),
}

pub fn cliamp_binary() -> Option<String> {
    std::process::Command::new("cliamp")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .ok()
        .filter(|s| s.success())
        .map(|_| "cliamp".to_string())
}

pub fn try_connect() -> Result<UnixStream> {
    let path = socket_path();
    UnixStream::connect(&path).with_context(|| format!("connect to {}", path.display()))
}

pub struct DaemonChild {
    child: Child,
    stderr_lines: Vec<String>,
}

impl DaemonChild {
    pub fn spawn() -> Result<Self> {
        let child = Command::new("cliamp")
            .arg("--daemon")
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .context("spawn cliamp --daemon")?;
        Ok(Self {
            child,
            stderr_lines: Vec::new(),
        })
    }

    pub fn poll_stderr(&mut self) {
        if let Some(stderr) = self.child.stderr.take() {
            let reader = BufReader::new(stderr);
            for line in reader.lines().map_while(Result::ok) {
                self.stderr_lines.push(line);
                if self.stderr_lines.len() > 20 {
                    self.stderr_lines.remove(0);
                }
            }
        }
    }

    pub fn exited(&mut self) -> Option<String> {
        self.poll_stderr();
        match self.child.try_wait() {
            Ok(Some(status)) => {
                let msg = if self.stderr_lines.is_empty() {
                    format!("cliamp --daemon exited with {}", status)
                } else {
                    format!(
                        "cliamp --daemon exited with {}: {}",
                        status,
                        self.stderr_lines.join("; ")
                    )
                };
                Some(msg)
            }
            Ok(None) => None,
            Err(e) => Some(e.to_string()),
        }
    }

    pub fn stderr_tail(&self) -> String {
        self.stderr_lines.join("\n")
    }
}

pub fn wait_for_socket(timeout: Duration) -> Result<UnixStream> {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if let Ok(stream) = try_connect() {
            return Ok(stream);
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    anyhow::bail!(
        "timed out waiting for socket at {}",
        socket_path().display()
    )
}
