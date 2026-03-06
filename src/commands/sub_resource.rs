use anyhow::{bail, Result};
use colored::Colorize;
use serde::Serialize;
use std::path::Path;

use crate::output;
use crate::project_util;
use crate::scene_parser;

// --- sub_resource add ---

#[derive(Serialize)]
pub struct SubResourceAddReport {
    pub scene: String,
    pub sub_resource_id: String,
    pub resource_type: String,
    pub properties: Vec<PropEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wired_to: Option<WireEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

#[derive(Serialize)]
pub struct PropEntry {
    pub key: String,
    pub value: String,
}

#[derive(Serialize)]
pub struct WireEntry {
    pub node: String,
    pub property: String,
}

pub fn run_add(
    scene_path: &str,
    resource_type: &str,
    props: &[(String, String)],
    wire_node: Option<&str>,
    wire_property: Option<&str>,
    json_mode: bool,
) -> Result<bool> {
    project_util::ensure_project_context(Some(Path::new(scene_path)))?;
    let path = Path::new(scene_path);
    if !path.is_file() {
        bail!("Scene file not found: {}", scene_path);
    }
    scene_parser::require_scene_file(path)?;

    // Validate: wire_node and wire_property must both be provided or both absent
    if wire_node.is_some() != wire_property.is_some() {
        bail!("Both wire_node and wire_property must be provided together");
    }

    let sub_id = scene_parser::add_sub_resource_to_file(
        path,
        resource_type,
        props,
        wire_node,
        wire_property,
    )?;

    let warning = if wire_node.is_none() {
        Some("Sub-resource created without wiring to any node. Use --wire-node/--wire-property or scene edit to connect it.".to_string())
    } else {
        None
    };

    if json_mode {
        let report = SubResourceAddReport {
            scene: scene_path.to_string(),
            sub_resource_id: sub_id.clone(),
            resource_type: resource_type.to_string(),
            properties: props
                .iter()
                .map(|(k, v)| PropEntry {
                    key: k.clone(),
                    value: v.clone(),
                })
                .collect(),
            wired_to: wire_node.map(|node| WireEntry {
                node: node.to_string(),
                property: wire_property.unwrap_or("").to_string(),
            }),
            warning: warning.clone(),
        };
        let envelope = output::JsonEnvelope {
            ok: true,
            command: "sub_resource add".into(),
            data: Some(report),
            error: None,
        };
        output::emit_json(&envelope);
    } else {
        println!(
            "  \u{2713} Added sub_resource '{}' (type: {}) to {}",
            sub_id, resource_type, scene_path
        );
        for (k, v) in props {
            println!("    {} = {}", k, v);
        }
        if let (Some(node), Some(prop)) = (wire_node, wire_property) {
            println!("    wired to {}.{}", node, prop);
        }
        if let Some(ref warn) = warning {
            println!("  {} {}", "warning:".yellow(), warn);
        }
    }

    Ok(true)
}

// --- sub_resource edit ---

#[derive(Serialize)]
pub struct SubResourceEditReport {
    pub scene: String,
    pub sub_resource_id: String,
    pub edits: Vec<EditEntry>,
}

#[derive(Serialize)]
pub struct EditEntry {
    pub property: String,
    pub value: String,
}

pub fn run_edit(
    scene_path: &str,
    sub_id: &str,
    edits: &[String],
    json_mode: bool,
) -> Result<bool> {
    project_util::ensure_project_context(Some(Path::new(scene_path)))?;
    let path = Path::new(scene_path);
    if !path.is_file() {
        bail!("Scene file not found: {}", scene_path);
    }
    scene_parser::require_scene_file(path)?;

    let mut edit_entries = Vec::new();

    for edit_arg in edits {
        let parts: Vec<&str> = edit_arg.splitn(2, '=').collect();
        if parts.len() != 2 {
            bail!(
                "Invalid --set format: '{}'\nExpected: property=value",
                edit_arg
            );
        }
        let property = parts[0];
        let value = scene_parser::format_prop_value(parts[1]);

        scene_parser::edit_sub_resource_property(path, sub_id, property, &value)?;

        edit_entries.push(EditEntry {
            property: property.to_string(),
            value,
        });
    }

    if json_mode {
        let report = SubResourceEditReport {
            scene: scene_path.to_string(),
            sub_resource_id: sub_id.to_string(),
            edits: edit_entries,
        };
        let envelope = output::JsonEnvelope {
            ok: true,
            command: "sub_resource edit".into(),
            data: Some(report),
            error: None,
        };
        output::emit_json(&envelope);
    } else {
        println!(
            "  \u{2713} Edited sub_resource '{}' in {}",
            sub_id, scene_path
        );
        for edit in &edit_entries {
            println!("    {} = {}", edit.property, edit.value);
        }
    }

    Ok(true)
}
