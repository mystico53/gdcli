use anyhow::{bail, Result};
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

const MAX_SESSIONS: usize = 4;
const STALE_TIMEOUT_SECS: u64 = 300; // 5 minutes after process exits

static SESSIONS: OnceLock<Mutex<HashMap<String, RunSession>>> = OnceLock::new();
static COUNTER: AtomicU64 = AtomicU64::new(1);

fn sessions() -> &'static Mutex<HashMap<String, RunSession>> {
    SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Debug, Clone, PartialEq)]
pub enum SessionStatus {
    Running,
    Exited(i32),
    TimedOut,
    Killed,
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionStatus::Running => write!(f, "running"),
            SessionStatus::Exited(code) => write!(f, "exited({})", code),
            SessionStatus::TimedOut => write!(f, "timed_out"),
            SessionStatus::Killed => write!(f, "killed"),
        }
    }
}

struct SessionBuffer {
    data: Vec<u8>,
    cursor: usize,
}

impl SessionBuffer {
    fn new() -> Self {
        Self {
            data: Vec::new(),
            cursor: 0,
        }
    }

    fn read_new(&mut self) -> String {
        if self.cursor >= self.data.len() {
            return String::new();
        }
        let new_bytes = &self.data[self.cursor..];
        self.cursor = self.data.len();
        String::from_utf8_lossy(new_bytes).to_string()
    }

    fn read_all(&self) -> String {
        String::from_utf8_lossy(&self.data).to_string()
    }
}

struct RunSession {
    child: Child,
    stdout_buf: Arc<Mutex<SessionBuffer>>,
    stderr_buf: Arc<Mutex<SessionBuffer>>,
    status: SessionStatus,
    timeout_secs: u64,
    started_at: Instant,
    finished_at: Option<Instant>,
}

pub struct SessionReadResult {
    pub status: SessionStatus,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

/// Clean up stale sessions: kill timed-out processes, remove sessions idle too long after exit.
pub fn cleanup_sessions() {
    let mut map = match sessions().lock() {
        Ok(m) => m,
        Err(_) => return,
    };

    let now = Instant::now();
    let mut to_remove = Vec::new();

    for (id, session) in map.iter_mut() {
        match session.status {
            SessionStatus::Running => {
                // Check timeout
                if now.duration_since(session.started_at).as_secs() > session.timeout_secs {
                    let _ = session.child.kill();
                    let _ = session.child.wait();
                    session.status = SessionStatus::TimedOut;
                    session.finished_at = Some(now);
                }
            }
            SessionStatus::Exited(_) | SessionStatus::TimedOut | SessionStatus::Killed => {
                // Remove stale finished sessions
                if let Some(finished) = session.finished_at {
                    if now.duration_since(finished).as_secs() > STALE_TIMEOUT_SECS {
                        to_remove.push(id.clone());
                    }
                }
            }
        }
    }

    for id in to_remove {
        map.remove(&id);
    }
}

/// Start a new Godot session in the background.
pub fn start_session(godot_path: &Path, scene: Option<&str>, timeout_secs: u64) -> Result<String> {
    cleanup_sessions();

    let mut map = sessions()
        .lock()
        .map_err(|_| anyhow::anyhow!("Session lock poisoned"))?;

    // Count active sessions
    let active = map
        .values()
        .filter(|s| s.status == SessionStatus::Running)
        .count();
    if active >= MAX_SESSIONS {
        bail!(
            "Maximum {} concurrent sessions reached. Stop an existing session first.",
            MAX_SESSIONS
        );
    }

    let mut args = vec!["--headless".to_string()];
    if let Some(scene_path) = scene {
        args.push("--scene".to_string());
        args.push(scene_path.to_string());
    }

    let mut child = Command::new(godot_path)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to spawn Godot: {}", e))?;

    let stdout_buf = Arc::new(Mutex::new(SessionBuffer::new()));
    let stderr_buf = Arc::new(Mutex::new(SessionBuffer::new()));

    // Spawn background reader threads
    if let Some(stdout_pipe) = child.stdout.take() {
        let buf = Arc::clone(&stdout_buf);
        std::thread::spawn(move || {
            read_pipe_into_buffer(stdout_pipe, buf);
        });
    }

    if let Some(stderr_pipe) = child.stderr.take() {
        let buf = Arc::clone(&stderr_buf);
        std::thread::spawn(move || {
            read_pipe_into_buffer(stderr_pipe, buf);
        });
    }

    let id = format!("session_{}", COUNTER.fetch_add(1, Ordering::SeqCst));

    map.insert(
        id.clone(),
        RunSession {
            child,
            stdout_buf,
            stderr_buf,
            status: SessionStatus::Running,
            timeout_secs,
            started_at: Instant::now(),
            finished_at: None,
        },
    );

    Ok(id)
}

/// Read incremental output from a session.
pub fn read_session(session_id: &str) -> Result<SessionReadResult> {
    cleanup_sessions();

    let mut map = sessions()
        .lock()
        .map_err(|_| anyhow::anyhow!("Session lock poisoned"))?;

    let session = map
        .get_mut(session_id)
        .ok_or_else(|| anyhow::anyhow!("Session '{}' not found", session_id))?;

    // Check if process has exited
    if session.status == SessionStatus::Running {
        match session.child.try_wait() {
            Ok(Some(exit_status)) => {
                let code = exit_status.code().unwrap_or(-1);
                session.status = SessionStatus::Exited(code);
                session.finished_at = Some(Instant::now());
            }
            Ok(None) => {
                // Still running — check timeout
                if Instant::now().duration_since(session.started_at).as_secs()
                    > session.timeout_secs
                {
                    let _ = session.child.kill();
                    let _ = session.child.wait();
                    session.status = SessionStatus::TimedOut;
                    session.finished_at = Some(Instant::now());
                }
            }
            Err(_) => {}
        }
    }

    // Small delay to let reader threads catch up after exit
    if session.status != SessionStatus::Running {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    let stdout = session
        .stdout_buf
        .lock()
        .map(|mut b| b.read_new())
        .unwrap_or_default();
    let stderr = session
        .stderr_buf
        .lock()
        .map(|mut b| b.read_new())
        .unwrap_or_default();

    let exit_code = match &session.status {
        SessionStatus::Exited(code) => Some(*code),
        SessionStatus::TimedOut | SessionStatus::Killed => Some(-1),
        SessionStatus::Running => None,
    };

    Ok(SessionReadResult {
        status: session.status.clone(),
        stdout,
        stderr,
        exit_code,
    })
}

/// Stop a session: kill the process if running, return all output.
pub fn stop_session(session_id: &str) -> Result<SessionReadResult> {
    let mut map = sessions()
        .lock()
        .map_err(|_| anyhow::anyhow!("Session lock poisoned"))?;

    let session = map
        .get_mut(session_id)
        .ok_or_else(|| anyhow::anyhow!("Session '{}' not found", session_id))?;

    // Kill if still running
    if session.status == SessionStatus::Running {
        let _ = session.child.kill();
        let _ = session.child.wait();
        session.status = SessionStatus::Killed;
        session.finished_at = Some(Instant::now());
    }

    // Wait for reader threads to finish draining
    std::thread::sleep(std::time::Duration::from_millis(100));

    let stdout = session
        .stdout_buf
        .lock()
        .map(|b| b.read_all())
        .unwrap_or_default();
    let stderr = session
        .stderr_buf
        .lock()
        .map(|b| b.read_all())
        .unwrap_or_default();

    let exit_code = match &session.status {
        SessionStatus::Exited(code) => Some(*code),
        SessionStatus::TimedOut | SessionStatus::Killed => Some(-1),
        SessionStatus::Running => None,
    };

    let status = session.status.clone();

    // Remove the session from the map
    map.remove(session_id);

    Ok(SessionReadResult {
        status,
        stdout,
        stderr,
        exit_code,
    })
}

fn read_pipe_into_buffer<R: Read + Send + 'static>(mut pipe: R, buf: Arc<Mutex<SessionBuffer>>) {
    let mut chunk = [0u8; 4096];
    loop {
        match pipe.read(&mut chunk) {
            Ok(0) => break, // EOF
            Ok(n) => {
                if let Ok(mut b) = buf.lock() {
                    b.data.extend_from_slice(&chunk[..n]);
                }
            }
            Err(_) => break,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_status_display() {
        assert_eq!(SessionStatus::Running.to_string(), "running");
        assert_eq!(SessionStatus::Exited(0).to_string(), "exited(0)");
        assert_eq!(SessionStatus::TimedOut.to_string(), "timed_out");
        assert_eq!(SessionStatus::Killed.to_string(), "killed");
    }

    #[test]
    fn test_session_buffer_incremental_read() {
        let mut buf = SessionBuffer::new();
        buf.data.extend_from_slice(b"hello ");
        assert_eq!(buf.read_new(), "hello ");
        buf.data.extend_from_slice(b"world");
        assert_eq!(buf.read_new(), "world");
        assert_eq!(buf.read_new(), "");
        assert_eq!(buf.read_all(), "hello world");
    }
}
