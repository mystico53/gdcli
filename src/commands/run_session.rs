use anyhow::Result;
use serde::Serialize;

use crate::godot_finder;
use crate::output;
use crate::project_util;
use crate::session;

// --- run_start ---

#[derive(Serialize)]
pub struct RunStartReport {
    pub session_id: String,
    pub scene: Option<String>,
    pub timeout: u64,
}

pub fn run_start(timeout: u64, scene: Option<&str>, json_mode: bool) -> Result<bool> {
    project_util::ensure_project_context(scene.map(std::path::Path::new))?;
    let godot_info = godot_finder::find_and_probe()?;

    let session_id = session::start_session(&godot_info.path, scene, timeout)?;

    if json_mode {
        let report = RunStartReport {
            session_id,
            scene: scene.map(String::from),
            timeout,
        };
        let envelope = output::JsonEnvelope {
            ok: true,
            command: "run_start".into(),
            data: Some(report),
            error: None,
        };
        output::emit_json(&envelope);
    } else {
        println!("  \u{2713} Started session (timeout: {}s)", timeout);
    }

    Ok(true)
}

// --- run_read ---

#[derive(Serialize)]
pub struct RunReadReport {
    pub session_id: String,
    pub status: String,
    pub stdout: String,
    pub stderr: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
}

pub fn run_read(session_id: &str, json_mode: bool) -> Result<bool> {
    let read = session::read_session(session_id)?;

    if json_mode {
        let report = RunReadReport {
            session_id: session_id.to_string(),
            status: read.status.to_string(),
            stdout: read.stdout,
            stderr: read.stderr,
            exit_code: read.exit_code,
        };
        let envelope = output::JsonEnvelope {
            ok: true,
            command: "run_read".into(),
            data: Some(report),
            error: None,
        };
        output::emit_json(&envelope);
    } else {
        println!("Session: {} ({})", session_id, read.status);
        if !read.stdout.is_empty() {
            print!("{}", read.stdout);
        }
        if !read.stderr.is_empty() {
            eprint!("{}", read.stderr);
        }
    }

    Ok(true)
}

// --- run_stop ---

#[derive(Serialize)]
pub struct RunStopReport {
    pub session_id: String,
    pub status: String,
    pub stdout: String,
    pub stderr: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
}

pub fn run_stop(session_id: &str, json_mode: bool) -> Result<bool> {
    let stop = session::stop_session(session_id)?;

    if json_mode {
        let report = RunStopReport {
            session_id: session_id.to_string(),
            status: stop.status.to_string(),
            stdout: stop.stdout,
            stderr: stop.stderr,
            exit_code: stop.exit_code,
        };
        let envelope = output::JsonEnvelope {
            ok: true,
            command: "run_stop".into(),
            data: Some(report),
            error: None,
        };
        output::emit_json(&envelope);
    } else {
        println!("Session {} stopped ({})", session_id, stop.status);
        if !stop.stdout.is_empty() {
            print!("{}", stop.stdout);
        }
        if !stop.stderr.is_empty() {
            eprint!("{}", stop.stderr);
        }
    }

    Ok(true)
}
