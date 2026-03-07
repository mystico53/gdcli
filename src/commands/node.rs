use anyhow::{bail, Result};
use serde::Serialize;
use serde_json::Value;
use std::path::Path;

use crate::output;
use crate::project_util;
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_resource: Option<SubResourceInfo>,
}

#[derive(Serialize)]
pub struct SubResourceInfo {
    pub id: String,
    pub resource_type: String,
    pub wire_property: String,
    pub properties: Vec<PropEntry>,
}

fn infer_wire_property(node_type: &str) -> Option<&'static str> {
    match node_type {
        "CollisionShape2D" | "CollisionShape3D" => Some("shape"),
        "MeshInstance2D" | "MeshInstance3D" => Some("mesh"),
        "Sprite2D" | "Sprite3D" => Some("texture"),
        "AudioStreamPlayer" | "AudioStreamPlayer2D" | "AudioStreamPlayer3D" => Some("stream"),
        "Path2D" | "Path3D" => Some("curve"),
        "NavigationRegion2D" | "NavigationRegion3D" => Some("navigation_polygon"),
        _ => None,
    }
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
    sub_resource_type: Option<&str>,
    sub_resource_props: &[(String, String)],
    sub_resource_property: Option<&str>,
    json_mode: bool,
) -> Result<bool> {
    project_util::ensure_project_context(Some(Path::new(scene_path)))?;
    let path = Path::new(scene_path);
    if !path.is_file() {
        bail!("Scene file not found: {}", scene_path);
    }
    scene_parser::require_scene_file(path)?;

    if node_type == Some("PackedScene") {
        bail!(
            "PackedScene is not a node type — it's a resource.\n\
             To instance a scene, use --instance instead:\n  \
             gdcli node add <scene> <Name> --instance res://path/to/scene.tscn"
        );
    }

    // Handle inline sub_resource creation
    let mut all_props = props.to_vec();
    let mut sub_info: Option<SubResourceInfo> = None;

    if let Some(sr_type) = sub_resource_type {
        let wire_prop = sub_resource_property
            .map(String::from)
            .or_else(|| node_type.and_then(infer_wire_property).map(String::from))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Cannot infer wire property for node type '{}'. Use --sub-resource-property to specify it.",
                    node_type.unwrap_or("(none)")
                )
            })?;

        let sub_id =
            scene_parser::add_sub_resource_to_file(path, sr_type, sub_resource_props, None, None)?;

        all_props.push((wire_prop.clone(), format!("SubResource(\"{}\")", sub_id)));

        sub_info = Some(SubResourceInfo {
            id: sub_id,
            resource_type: sr_type.to_string(),
            wire_property: wire_prop,
            properties: sub_resource_props
                .iter()
                .map(|(k, v)| PropEntry {
                    key: k.clone(),
                    value: v.clone(),
                })
                .collect(),
        });
    }

    scene_parser::add_node_to_file(
        path, node_type, node_name, parent, script, &all_props, instance,
    )?;

    let parent_display = parent.unwrap_or(".").to_string();

    if json_mode {
        let report = NodeAddReport {
            scene: scene_path.to_string(),
            node_name: node_name.to_string(),
            node_type: node_type.map(String::from),
            parent: parent_display.clone(),
            script: script.map(String::from),
            instance: instance.map(String::from),
            properties: all_props
                .iter()
                .map(|(k, v)| PropEntry {
                    key: k.clone(),
                    value: v.clone(),
                })
                .collect(),
            sub_resource: sub_info,
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
            node_name,
            node_type.unwrap_or("?"),
            scene_path,
            parent_display
        );
        if let Some(s) = script {
            println!("    script: {}", s);
        }
        for (k, v) in &all_props {
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
    project_util::ensure_project_context(Some(Path::new(scene_path)))?;
    let path = Path::new(scene_path);
    if !path.is_file() {
        bail!("Scene file not found: {}", scene_path);
    }
    scene_parser::require_scene_file(path)?;

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

// --- node reorder ---

#[derive(Serialize)]
pub struct NodeReorderReport {
    pub scene: String,
    pub node: String,
    pub moved: bool,
}

pub fn run_reorder(
    scene_path: &str,
    node_name: &str,
    position: Option<&str>,
    before: Option<&str>,
    after: Option<&str>,
    json_mode: bool,
) -> Result<bool> {
    project_util::ensure_project_context(Some(Path::new(scene_path)))?;
    let path = Path::new(scene_path);
    if !path.is_file() {
        bail!("Scene file not found: {}", scene_path);
    }
    scene_parser::require_scene_file(path)?;

    let pos = position.map(|p| p.parse::<usize>()).transpose().map_err(|_| {
        anyhow::anyhow!("position must be a non-negative integer")
    })?;

    if pos.is_none() && before.is_none() && after.is_none() {
        bail!("Must specify one of: position, before, or after");
    }

    scene_parser::reorder_node_in_file(path, node_name, pos, before, after)?;

    if json_mode {
        let report = NodeReorderReport {
            scene: scene_path.to_string(),
            node: node_name.to_string(),
            moved: true,
        };
        let envelope = output::JsonEnvelope {
            ok: true,
            command: "node reorder".into(),
            data: Some(report),
            error: None,
        };
        output::emit_json(&envelope);
    } else {
        println!(
            "  \u{2713} Reordered node '{}' in {}",
            node_name, scene_path
        );
    }

    Ok(true)
}

// --- node add_many ---

#[derive(Serialize)]
pub struct NodeAddManyReport {
    pub scene: String,
    pub added: Vec<String>,
    pub count: usize,
}

pub fn run_add_many(
    scene_path: &str,
    nodes: &[Value],
    json_mode: bool,
) -> Result<bool> {
    project_util::ensure_project_context(Some(Path::new(scene_path)))?;
    let path = Path::new(scene_path);
    if !path.is_file() {
        bail!("Scene file not found: {}", scene_path);
    }
    scene_parser::require_scene_file(path)?;

    let mut added_names = Vec::new();

    for node_def in nodes {
        let name = node_def
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Each node must have a 'name' field"))?;
        let node_type = node_def.get("node_type").and_then(|v| v.as_str());
        let parent = node_def.get("parent").and_then(|v| v.as_str());
        let script = node_def.get("script").and_then(|v| v.as_str());
        let instance = node_def.get("instance").and_then(|v| v.as_str());

        if node_type.is_none() && instance.is_none() {
            bail!("Node '{}': either node_type or instance must be provided", name);
        }

        let props_raw: Vec<String> = node_def
            .get("props")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let parsed_props: Vec<(String, String)> = props_raw
            .iter()
            .filter_map(|p| {
                let parts: Vec<&str> = p.splitn(2, '=').collect();
                if parts.len() == 2 {
                    Some((
                        parts[0].to_string(),
                        scene_parser::format_prop_value(parts[1]),
                    ))
                } else {
                    None
                }
            })
            .collect();

        // Handle inline sub_resource
        let sub_resource_type = node_def.get("sub_resource_type").and_then(|v| v.as_str());
        let sub_resource_property = node_def.get("sub_resource_property").and_then(|v| v.as_str());
        let sub_props_raw: Vec<String> = node_def
            .get("sub_resource_props")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let parsed_sub_props: Vec<(String, String)> = sub_props_raw
            .iter()
            .filter_map(|p| {
                let parts: Vec<&str> = p.splitn(2, '=').collect();
                if parts.len() == 2 {
                    Some((
                        parts[0].to_string(),
                        scene_parser::format_prop_value(parts[1]),
                    ))
                } else {
                    None
                }
            })
            .collect();

        let mut all_props = parsed_props;

        if let Some(sr_type) = sub_resource_type {
            let wire_prop = sub_resource_property
                .map(String::from)
                .or_else(|| node_type.and_then(infer_wire_property).map(String::from))
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Cannot infer wire property for node '{}' type '{}'. Use sub_resource_property.",
                        name,
                        node_type.unwrap_or("(none)")
                    )
                })?;

            let sub_id =
                scene_parser::add_sub_resource_to_file(path, sr_type, &parsed_sub_props, None, None)?;
            all_props.push((wire_prop, format!("SubResource(\"{}\")", sub_id)));
        }

        scene_parser::add_node_to_file(
            path, node_type, name, parent, script, &all_props, instance,
        )?;

        added_names.push(name.to_string());
    }

    let count = added_names.len();

    if json_mode {
        let report = NodeAddManyReport {
            scene: scene_path.to_string(),
            added: added_names,
            count,
        };
        let envelope = output::JsonEnvelope {
            ok: true,
            command: "node add_many".into(),
            data: Some(report),
            error: None,
        };
        output::emit_json(&envelope);
    } else {
        println!(
            "  \u{2713} Added {} node(s) to {}",
            count, scene_path
        );
    }

    Ok(true)
}
