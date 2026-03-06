use anyhow::{bail, Result};
use serde::Serialize;
use std::fs;
use std::path::Path;

use crate::output;
use crate::project_util;
use crate::scene_parser;

// --- scene create ---

#[derive(Serialize)]
pub struct SceneCreateReport {
    pub path: String,
    pub root_type: String,
    pub uid: String,
}

pub fn run_create(scene_path: &str, root_type: &str, root_name: Option<&str>, script: Option<&str>, force: bool, json_mode: bool) -> Result<bool> {
    project_util::ensure_project_context(Some(Path::new(scene_path)))?;
    let path = Path::new(scene_path);

    if path.is_file() && !force {
        bail!(
            "File already exists: {}\nUse --force to overwrite.",
            scene_path
        );
    }

    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    // Derive root node name from filename if not explicitly provided
    let resolved_name = match root_name {
        Some(name) => name.to_string(),
        None => scene_parser::filename_to_node_name(scene_path),
    };

    let uid = scene_parser::generate_uid();
    let content = scene_parser::generate_minimal_scene(root_type, &resolved_name, &uid, script);

    scene_parser::atomic_write(path, &content)?;

    if json_mode {
        let report = SceneCreateReport {
            path: scene_path.to_string(),
            root_type: root_type.to_string(),
            uid: uid.clone(),
        };
        let envelope = output::JsonEnvelope {
            ok: true,
            command: "scene create".into(),
            data: Some(report),
            error: None,
        };
        output::emit_json(&envelope);
    } else {
        println!(
            "  \u{2713} Created {} (root: {} \"{}\", uid: {})",
            scene_path, root_type, resolved_name, uid
        );
    }

    Ok(true)
}

// --- scene list ---

#[derive(Serialize)]
pub struct SceneListReport {
    pub scenes: Vec<SceneEntry>,
    pub total: usize,
}

#[derive(Serialize)]
pub struct SceneEntry {
    pub path: String,
    pub node_count: usize,
    pub ext_resource_count: usize,
}

pub fn run_list(json_mode: bool) -> Result<bool> {
    project_util::ensure_project_context(None)?;

    let scene_files = scene_parser::find_scene_files(Path::new("."));
    let mut scenes = Vec::new();

    for file_path in &scene_files {
        let display_path = file_path
            .strip_prefix(".")
            .unwrap_or(file_path)
            .display()
            .to_string()
            .replace('\\', "/");

        match scene_parser::parse_scene(file_path) {
            Ok(parsed) => {
                scenes.push(SceneEntry {
                    path: display_path,
                    node_count: parsed.nodes.len(),
                    ext_resource_count: parsed.ext_resources.len(),
                });
            }
            Err(_) => {
                scenes.push(SceneEntry {
                    path: display_path,
                    node_count: 0,
                    ext_resource_count: 0,
                });
            }
        }
    }

    let total = scenes.len();

    if json_mode {
        let report = SceneListReport { scenes, total };
        let envelope = output::JsonEnvelope {
            ok: true,
            command: "scene list".into(),
            data: Some(report),
            error: None,
        };
        output::emit_json(&envelope);
    } else {
        output::print_header(&format!("{} scene(s) found", total));
        for scene in &scenes {
            println!(
                "  {} ({} nodes, {} resources)",
                scene.path, scene.node_count, scene.ext_resource_count
            );
        }
    }

    Ok(true)
}

// --- scene validate ---

#[derive(Serialize)]
pub struct ValidateReport {
    pub scene: String,
    pub issues: Vec<ValidationIssue>,
    pub issue_count: usize,
}

#[derive(Serialize)]
pub struct ValidationIssue {
    pub severity: String,
    pub message: String,
}

pub fn run_validate(scene_path: &str, json_mode: bool) -> Result<bool> {
    let path = Path::new(scene_path);
    if !path.is_file() {
        bail!("Scene file not found: {}", scene_path);
    }
    scene_parser::require_scene_file(path)?;
    project_util::ensure_project_context(Some(path))?;

    let parsed = scene_parser::parse_scene(path)?;
    let mut issues = Vec::new();

    // Check external resource paths
    for ext_res in &parsed.ext_resources {
        let res_path = ext_res.path.strip_prefix("res://").unwrap_or(&ext_res.path);
        if !Path::new(res_path).is_file() {
            issues.push(ValidationIssue {
                severity: "error".into(),
                message: format!(
                    "Missing external resource: {} (type: {}, id: {})",
                    ext_res.path, ext_res.resource_type, ext_res.id
                ),
            });
        }
    }

    // Check for nodes without a type (skip instanced scene nodes — they legitimately lack a type)
    for node in &parsed.nodes {
        if node.node_type.is_empty() && node.parent.is_some() && node.instance.is_none() {
            issues.push(ValidationIssue {
                severity: "warning".into(),
                message: format!(
                    "Node '{}' has no type (may be an instanced scene)",
                    node.name
                ),
            });
        }
    }

    let issue_count = issues.len();
    let clean = issue_count == 0;

    if json_mode {
        let report = ValidateReport {
            scene: scene_path.to_string(),
            issues,
            issue_count,
        };
        let envelope = output::JsonEnvelope {
            ok: clean,
            command: "scene validate".into(),
            data: Some(report),
            error: if clean {
                None
            } else {
                Some(format!("{} issue(s) found", issue_count))
            },
        };
        output::emit_json(&envelope);
    } else if clean {
        println!("  \u{2713} {} — no issues", scene_path);
    } else {
        output::print_header(&format!("{} — {} issue(s):", scene_path, issue_count));
        for issue in &issues {
            let icon = if issue.severity == "error" {
                "\u{2717}"
            } else {
                "!"
            };
            println!("  {} {}", icon, issue.message);
        }
    }

    Ok(clean)
}

// --- scene inspect ---

#[derive(Serialize)]
pub struct SceneInspectReport {
    pub scene: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_filter: Option<String>,
    pub ext_resources: Vec<InspectExtResource>,
    pub sub_resources: Vec<InspectSubResource>,
    pub nodes: Vec<InspectNode>,
    pub connections: Vec<InspectConnection>,
}

#[derive(Serialize)]
pub struct InspectExtResource {
    pub id: String,
    #[serde(rename = "type")]
    pub resource_type: String,
    pub path: String,
}

#[derive(Serialize)]
pub struct InspectSubResource {
    pub id: String,
    #[serde(rename = "type")]
    pub resource_type: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub properties: Vec<InspectProperty>,
}

#[derive(Serialize)]
pub struct InspectNode {
    pub name: String,
    #[serde(rename = "type")]
    pub node_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub properties: Vec<InspectProperty>,
}

#[derive(Serialize)]
pub struct InspectProperty {
    pub key: String,
    pub value: String,
}

#[derive(Serialize)]
pub struct InspectConnection {
    pub signal: String,
    pub from: String,
    pub to: String,
    pub method: String,
}

/// Extract SubResource("id") and ExtResource("id") references from a property value string.
fn extract_resource_refs(value: &str) -> (Vec<String>, Vec<String>) {
    let mut sub_ids = Vec::new();
    let mut ext_ids = Vec::new();
    for cap in value.match_indices("SubResource(\"") {
        let start = cap.0 + "SubResource(\"".len();
        if let Some(end) = value[start..].find("\")") {
            sub_ids.push(value[start..start + end].to_string());
        }
    }
    for cap in value.match_indices("ExtResource(\"") {
        let start = cap.0 + "ExtResource(\"".len();
        if let Some(end) = value[start..].find("\")") {
            ext_ids.push(value[start..start + end].to_string());
        }
    }
    (sub_ids, ext_ids)
}

pub fn run_inspect(scene_path: &str, node_filter: Option<&str>, json_mode: bool) -> Result<bool> {
    project_util::ensure_project_context(Some(Path::new(scene_path)))?;
    let path = Path::new(scene_path);
    if !path.is_file() {
        bail!("Scene file not found: {}", scene_path);
    }
    scene_parser::require_scene_file(path)?;

    let parsed = scene_parser::parse_scene(path)?;

    // Apply node filter if provided
    let (filtered_nodes, filtered_sub_resources, filtered_ext_resources, filtered_connections) =
        if let Some(name) = node_filter {
            let target = parsed.nodes.iter().find(|n| n.name == name);
            if target.is_none() {
                bail!("Node '{}' not found in {}", name, scene_path);
            }
            let target = target.unwrap();

            // Collect resource refs from the node's properties + instance
            let mut sub_ids = Vec::new();
            let mut ext_ids = Vec::new();
            for prop in &target.properties {
                let (s, e) = extract_resource_refs(&prop.value);
                sub_ids.extend(s);
                ext_ids.extend(e);
            }
            if let Some(ref inst) = target.instance {
                let (_, e) = extract_resource_refs(inst);
                ext_ids.extend(e);
            }

            // Recursively collect refs from referenced sub_resources
            let mut i = 0;
            while i < sub_ids.len() {
                if let Some(sub) = parsed.sub_resources.iter().find(|s| s.id == sub_ids[i]) {
                    for prop in &sub.properties {
                        let (s, e) = extract_resource_refs(&prop.value);
                        for sid in s {
                            if !sub_ids.contains(&sid) {
                                sub_ids.push(sid);
                            }
                        }
                        ext_ids.extend(e);
                    }
                }
                i += 1;
            }

            let nodes: Vec<_> = vec![target.clone()];
            let subs: Vec<_> = parsed
                .sub_resources
                .iter()
                .filter(|s| sub_ids.contains(&s.id))
                .cloned()
                .collect();
            let exts: Vec<_> = parsed
                .ext_resources
                .iter()
                .filter(|e| ext_ids.contains(&e.id))
                .cloned()
                .collect();
            // Connections involving this node (by name or "." for root)
            let node_path = &target.name;
            let conns: Vec<_> = parsed
                .connections
                .iter()
                .filter(|c| c.from == *node_path || c.to == *node_path)
                .cloned()
                .collect();
            (nodes, subs, exts, conns)
        } else {
            (
                parsed.nodes.clone(),
                parsed.sub_resources.clone(),
                parsed.ext_resources.clone(),
                parsed.connections.clone(),
            )
        };

    if json_mode {
        let report = SceneInspectReport {
            scene: scene_path.to_string(),
            uid: parsed.uid.clone(),
            node_filter: node_filter.map(String::from),
            ext_resources: filtered_ext_resources
                .iter()
                .map(|e| InspectExtResource {
                    id: e.id.clone(),
                    resource_type: e.resource_type.clone(),
                    path: e.path.clone(),
                })
                .collect(),
            sub_resources: filtered_sub_resources
                .iter()
                .map(|s| InspectSubResource {
                    id: s.id.clone(),
                    resource_type: s.resource_type.clone(),
                    properties: s
                        .properties
                        .iter()
                        .map(|p| InspectProperty {
                            key: p.key.clone(),
                            value: p.value.clone(),
                        })
                        .collect(),
                })
                .collect(),
            nodes: filtered_nodes
                .iter()
                .map(|n| InspectNode {
                    name: n.name.clone(),
                    node_type: n.node_type.clone(),
                    parent: n.parent.clone(),
                    instance: n.instance.clone(),
                    properties: n
                        .properties
                        .iter()
                        .map(|p| InspectProperty {
                            key: p.key.clone(),
                            value: p.value.clone(),
                        })
                        .collect(),
                })
                .collect(),
            connections: filtered_connections
                .iter()
                .map(|c| InspectConnection {
                    signal: c.signal.clone(),
                    from: c.from.clone(),
                    to: c.to.clone(),
                    method: c.method.clone(),
                })
                .collect(),
        };
        let envelope = output::JsonEnvelope {
            ok: true,
            command: "scene inspect".into(),
            data: Some(report),
            error: None,
        };
        output::emit_json(&envelope);
    } else {
        output::print_header(&format!("Scene: {}", scene_path));
        if let Some(ref uid) = parsed.uid {
            println!("  UID: {}", uid);
        }
        if let Some(name) = node_filter {
            println!("  Filtered to node: {}", name);
        }

        if !filtered_ext_resources.is_empty() {
            println!();
            println!("  External Resources ({}):", filtered_ext_resources.len());
            for ext in &filtered_ext_resources {
                println!("    [{}] {} — {}", ext.id, ext.resource_type, ext.path);
            }
        }

        if !filtered_sub_resources.is_empty() {
            println!();
            println!("  Sub Resources ({}):", filtered_sub_resources.len());
            for sub in &filtered_sub_resources {
                println!("    [{}] {}", sub.id, sub.resource_type);
                for prop in &sub.properties {
                    println!("      {} = {}", prop.key, prop.value);
                }
            }
        }

        if !filtered_nodes.is_empty() {
            println!();
            println!("  Nodes ({}):", filtered_nodes.len());
            for node in &filtered_nodes {
                let parent_str = match &node.parent {
                    Some(p) => format!(" (parent: {})", p),
                    None => " (root)".to_string(),
                };
                if let Some(ref inst) = node.instance {
                    println!("    {} [instance] {}{}", node.name, inst, parent_str);
                } else {
                    println!("    {} [{}]{}", node.name, node.node_type, parent_str);
                }
                for prop in &node.properties {
                    println!("      {} = {}", prop.key, prop.value);
                }
            }
        }

        if !filtered_connections.is_empty() {
            println!();
            println!("  Connections ({}):", filtered_connections.len());
            for conn in &filtered_connections {
                println!(
                    "    {}.{} -> {}.{}",
                    conn.from, conn.signal, conn.to, conn.method
                );
            }
        }
    }

    Ok(true)
}

// --- scene edit ---

#[derive(Serialize)]
pub struct SceneEditReport {
    pub scene: String,
    pub edits: Vec<EditEntry>,
}

#[derive(Serialize)]
pub struct EditEntry {
    pub node: String,
    pub property: String,
    pub value: String,
}

pub fn run_edit(scene_path: &str, set_args: &[String], json_mode: bool) -> Result<bool> {
    project_util::ensure_project_context(Some(Path::new(scene_path)))?;
    let path = Path::new(scene_path);
    if !path.is_file() {
        bail!("Scene file not found: {}", scene_path);
    }
    scene_parser::require_scene_file(path)?;

    let mut edits = Vec::new();

    for set_arg in set_args {
        // Parse "NodeName::property=value"
        let parts: Vec<&str> = set_arg.splitn(2, "::").collect();
        if parts.len() != 2 {
            bail!(
                "Invalid --set format: '{}'\nExpected: NodeName::property=value",
                set_arg
            );
        }
        let node_name = parts[0];
        let prop_parts: Vec<&str> = parts[1].splitn(2, '=').collect();
        if prop_parts.len() != 2 {
            bail!(
                "Invalid --set format: '{}'\nExpected: NodeName::property=value",
                set_arg
            );
        }
        let property = prop_parts[0];
        let raw_value = prop_parts[1];

        // Resource-aware: if value is a res:// path, add an ext_resource and use ExtResource("id")
        let value = if raw_value.starts_with("res://") {
            let res_type = scene_parser::infer_resource_type(raw_value);
            let ext_id = scene_parser::add_ext_resource_to_file(path, raw_value, res_type)?;
            format!("ExtResource(\"{}\")", ext_id)
        } else {
            scene_parser::format_prop_value(raw_value)
        };

        scene_parser::edit_node_property(path, node_name, property, &value)?;

        edits.push(EditEntry {
            node: node_name.to_string(),
            property: property.to_string(),
            value,
        });
    }

    if json_mode {
        let report = SceneEditReport {
            scene: scene_path.to_string(),
            edits,
        };
        let envelope = output::JsonEnvelope {
            ok: true,
            command: "scene edit".into(),
            data: Some(report),
            error: None,
        };
        output::emit_json(&envelope);
    } else {
        println!("  \u{2713} Edited {}", scene_path);
        for edit in &edits {
            println!("    {}::{} = {}", edit.node, edit.property, edit.value);
        }
    }

    Ok(true)
}
