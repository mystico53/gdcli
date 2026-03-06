use anyhow::{Context, Result};
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;
use wait_timeout::ChildExt;

pub struct RunResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub timed_out: bool,
}

/// Run Godot with `--headless` prepended to the given args.
pub fn run(godot_path: &Path, args: &[&str], timeout_secs: u64) -> Result<RunResult> {
    let mut full_args = vec!["--headless"];
    full_args.extend_from_slice(args);
    run_raw(godot_path, &full_args, timeout_secs)
}

/// Run Godot with exactly the given args (no flags prepended).
pub fn run_raw(godot_path: &Path, args: &[&str], timeout_secs: u64) -> Result<RunResult> {
    let start = Instant::now();

    let mut child = Command::new(godot_path)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to spawn Godot at {}", godot_path.display()))?;

    // Take pipes BEFORE waiting — we'll read them in background threads
    // to avoid pipe buffer deadlocks (process blocks writing if nobody reads).
    let stdout_pipe = child.stdout.take();
    let stderr_pipe = child.stderr.take();

    let stdout_thread = std::thread::spawn(move || {
        let mut buf = String::new();
        if let Some(mut pipe) = stdout_pipe {
            let _ = pipe.read_to_string(&mut buf);
        }
        buf
    });

    let stderr_thread = std::thread::spawn(move || {
        let mut buf = String::new();
        if let Some(mut pipe) = stderr_pipe {
            let _ = pipe.read_to_string(&mut buf);
        }
        buf
    });

    let timeout = std::time::Duration::from_secs(timeout_secs);
    let status = child
        .wait_timeout(timeout)
        .context("failed waiting for Godot process")?;

    let duration_ms = start.elapsed().as_millis() as u64;

    let timed_out = status.is_none();
    if timed_out {
        let _ = child.kill();
        let _ = child.wait();
    }

    let stdout_buf = stdout_thread.join().unwrap_or_default();
    let stderr_buf = stderr_thread.join().unwrap_or_default();

    let exit_code = if timed_out {
        -1
    } else {
        status.and_then(|s| s.code()).unwrap_or(-1)
    };

    Ok(RunResult {
        exit_code,
        stdout: stdout_buf,
        stderr: stderr_buf,
        duration_ms,
        timed_out,
    })
}
