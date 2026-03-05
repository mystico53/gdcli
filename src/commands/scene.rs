use anyhow::{bail, Result};
use serde::Serialize;
use std::path::Path;

use crate::output;
use crate::scene_parser;

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
    if !Path::new("project.godot").is_file() {
        bail!(
            "project.godot not found in current directory.\n\
             Run this command from your Godot project root."
        );
    }

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

    // Check for nodes without a type (possible instance placeholder issues)
    for node in &parsed.nodes {
        if node.node_type.is_empty() && node.parent.is_some() {
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
