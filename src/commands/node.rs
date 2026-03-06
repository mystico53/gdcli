use anyhow::{bail, Result};
use serde::Serialize;
use std::path::Path;

use crate::output;
use crate::scene_parser;

// --- node add ---

#[derive(Serialize)]
pub struct NodeAddReport {
    pub scene: String,
    pub node_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_type: Option<String>,
    pub parent: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    pub properties: Vec<PropEntry>,
}

#[derive(Serialize)]
pub struct PropEntry {
    pub key: String,
    pub value: String,
}

#[allow(clippy::too_many_arguments)]
pub fn run_add(
    scene_path: &str,
    node_type: Option<&str>,
    node_name: &str,
    parent: Option<&str>,
    script: Option<&str>,
    props: &[(String, String)],
    instance: Option<&str>,
    json_mode: bool,
) -> Result<bool> {
    let path = Path::new(scene_path);
    if !path.is_file() {
        bail!("Scene file not found: {}", scene_path);
    }

    scene_parser::add_node_to_file(path, node_type, node_name, parent, script, props, instance)?;

    let parent_display = parent.unwrap_or(".").to_string();

    if json_mode {
        let report = NodeAddReport {
            scene: scene_path.to_string(),
            node_name: node_name.to_string(),
            node_type: node_type.map(String::from),
            parent: parent_display.clone(),
            script: script.map(String::from),
            instance: instance.map(String::from),
            properties: props
                .iter()
                .map(|(k, v)| PropEntry {
                    key: k.clone(),
                    value: v.clone(),
                })
                .collect(),
        };
        let envelope = output::JsonEnvelope {
            ok: true,
            command: "node add".into(),
            data: Some(report),
            error: None,
        };
        output::emit_json(&envelope);
    } else if let Some(inst) = instance {
        println!(
            "  \u{2713} Added instanced node '{}' ({}) to {} under '{}'",
            node_name, inst, scene_path, parent_display
        );
    } else {
        println!(
            "  \u{2713} Added node '{}' (type: {}) to {} under '{}'",
            node_name, node_type.unwrap_or("?"), scene_path, parent_display
        );
        if let Some(s) = script {
            println!("    script: {}", s);
        }
        for (k, v) in props {
            println!("    {} = {}", k, v);
        }
    }

    Ok(true)
}

// --- node remove ---

#[derive(Serialize)]
pub struct NodeRemoveReport {
    pub scene: String,
    pub removed: Vec<String>,
    pub removed_count: usize,
}

pub fn run_remove(scene_path: &str, node_name: &str, json_mode: bool) -> Result<bool> {
    let path = Path::new(scene_path);
    if !path.is_file() {
        bail!("Scene file not found: {}", scene_path);
    }

    let removed = scene_parser::remove_node_from_file(path, node_name)?;
    let removed_count = removed.len();

    if json_mode {
        let report = NodeRemoveReport {
            scene: scene_path.to_string(),
            removed,
            removed_count,
        };
        let envelope = output::JsonEnvelope {
            ok: true,
            command: "node remove".into(),
            data: Some(report),
            error: None,
        };
        output::emit_json(&envelope);
    } else {
        println!(
            "  \u{2713} Removed {} node(s) from {}",
            removed_count, scene_path
        );
    }

    Ok(true)
}
