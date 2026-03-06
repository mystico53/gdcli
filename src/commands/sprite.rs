use anyhow::{bail, Result};
use serde::Serialize;
use std::path::Path;

use crate::output;
use crate::project_util;
use crate::scene_parser;

#[derive(Serialize)]
pub struct LoadSpriteReport {
    pub scene: String,
    pub node_name: String,
    pub sprite_type: String,
    pub texture_path: String,
    pub ext_resource_id: String,
    pub parent: String,
    pub properties: Vec<PropEntry>,
}

#[derive(Serialize)]
pub struct PropEntry {
    pub key: String,
    pub value: String,
}

#[allow(clippy::too_many_arguments)]
pub fn run_load_sprite(
    scene_path: &str,
    node_name: &str,
    texture_path: &str,
    sprite_type: Option<&str>,
    parent: Option<&str>,
    props: &[(String, String)],
    json_mode: bool,
) -> Result<bool> {
    project_util::ensure_project_context(Some(Path::new(scene_path)))?;
    let path = Path::new(scene_path);
    if !path.is_file() {
        bail!("Scene file not found: {}", scene_path);
    }
    scene_parser::require_scene_file(path)?;

    // Validate texture file exists on disk
    let res_stripped = texture_path.strip_prefix("res://").unwrap_or(texture_path);
    if !Path::new(res_stripped).is_file() {
        bail!(
            "Texture file not found: {} (resolved to {})",
            texture_path,
            res_stripped
        );
    }

    let sprite_type = sprite_type.unwrap_or("Sprite2D");
    if sprite_type != "Sprite2D" && sprite_type != "Sprite3D" {
        bail!(
            "Invalid sprite_type '{}'. Must be Sprite2D or Sprite3D.",
            sprite_type
        );
    }

    // Infer resource type from extension
    let res_type = scene_parser::infer_resource_type(texture_path);

    // Add ext_resource (deduplicates if already present)
    let ext_id = scene_parser::add_ext_resource_to_file(path, texture_path, res_type)?;

    // Build props with texture prepended
    let texture_value = format!("ExtResource(\"{}\")", ext_id);
    let mut all_props = vec![("texture".to_string(), texture_value)];
    all_props.extend_from_slice(props);

    // Add the sprite node
    scene_parser::add_node_to_file(
        path,
        Some(sprite_type),
        node_name,
        parent,
        None,
        &all_props,
        None,
    )?;

    let parent_display = parent.unwrap_or(".").to_string();

    if json_mode {
        let report = LoadSpriteReport {
            scene: scene_path.to_string(),
            node_name: node_name.to_string(),
            sprite_type: sprite_type.to_string(),
            texture_path: texture_path.to_string(),
            ext_resource_id: ext_id,
            parent: parent_display.clone(),
            properties: all_props
                .iter()
                .map(|(k, v)| PropEntry {
                    key: k.clone(),
                    value: v.clone(),
                })
                .collect(),
        };
        let envelope = output::JsonEnvelope {
            ok: true,
            command: "load_sprite".into(),
            data: Some(report),
            error: None,
        };
        output::emit_json(&envelope);
    } else {
        println!(
            "  \u{2713} Added {} '{}' with texture {} to {} under '{}'",
            sprite_type, node_name, texture_path, scene_path, parent_display
        );
        for (k, v) in &all_props {
            println!("    {} = {}", k, v);
        }
    }

    Ok(true)
}
