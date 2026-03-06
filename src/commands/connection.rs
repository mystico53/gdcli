use anyhow::Result;
use serde::Serialize;
use std::path::Path;

use crate::output;
use crate::project_util;
use crate::scene_parser;

// --- connection add ---

#[derive(Serialize)]
pub struct ConnectionAddReport {
    pub scene: String,
    pub signal: String,
    pub from: String,
    pub to: String,
    pub method: String,
}

pub fn run_add(
    scene_path: &str,
    signal: &str,
    from: &str,
    to: &str,
    method: &str,
    json_mode: bool,
) -> Result<bool> {
    project_util::ensure_project_context(Some(Path::new(scene_path)))?;
    let path = Path::new(scene_path);
    scene_parser::require_scene_file(path)?;

    scene_parser::add_connection_to_file(path, signal, from, to, method)?;

    if json_mode {
        let report = ConnectionAddReport {
            scene: scene_path.to_string(),
            signal: signal.to_string(),
            from: from.to_string(),
            to: to.to_string(),
            method: method.to_string(),
        };
        let envelope = output::JsonEnvelope {
            ok: true,
            command: "connection add".into(),
            data: Some(report),
            error: None,
        };
        output::emit_json(&envelope);
    } else {
        println!(
            "  \u{2713} Connected {}.{} -> {}.{} in {}",
            from, signal, to, method, scene_path
        );
    }

    Ok(true)
}

// --- connection remove ---

#[derive(Serialize)]
pub struct ConnectionRemoveReport {
    pub scene: String,
    pub signal: String,
    pub from: String,
    pub to: String,
    pub method: String,
}

pub fn run_remove(
    scene_path: &str,
    signal: &str,
    from: &str,
    to: &str,
    method: &str,
    json_mode: bool,
) -> Result<bool> {
    project_util::ensure_project_context(Some(Path::new(scene_path)))?;
    let path = Path::new(scene_path);
    scene_parser::require_scene_file(path)?;

    scene_parser::remove_connection_from_file(path, signal, from, to, method)?;

    if json_mode {
        let report = ConnectionRemoveReport {
            scene: scene_path.to_string(),
            signal: signal.to_string(),
            from: from.to_string(),
            to: to.to_string(),
            method: method.to_string(),
        };
        let envelope = output::JsonEnvelope {
            ok: true,
            command: "connection remove".into(),
            data: Some(report),
            error: None,
        };
        output::emit_json(&envelope);
    } else {
        println!(
            "  \u{2713} Removed connection {}.{} -> {}.{} from {}",
            from, signal, to, method, scene_path
        );
    }

    Ok(true)
}
