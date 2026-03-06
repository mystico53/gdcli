use anyhow::Result;
use serde::Serialize;
use std::fs;
use std::path::Path;

use crate::output;
use crate::project_util;

#[derive(Serialize)]
pub struct ProjectInfo {
    pub name: String,
    pub main_scene: Option<String>,
    pub autoloads: Vec<String>,
    pub script_count: usize,
    pub scene_count: usize,
}

pub fn run_info(json_mode: bool) -> Result<bool> {
    project_util::ensure_project_context(None)?;

    let content = fs::read_to_string("project.godot")?;
    let info = parse_project_godot(&content);

    if json_mode {
        let envelope = output::JsonEnvelope {
            ok: true,
            command: "project info".into(),
            data: Some(&info),
            error: None,
        };
        output::emit_json(&envelope);
    } else {
        output::print_header("Project Info");
        println!("  Name:        {}", info.name);
        if let Some(ref scene) = info.main_scene {
            println!("  Main scene:  {}", scene);
        }
        if !info.autoloads.is_empty() {
            println!("  Autoloads:   {}", info.autoloads.join(", "));
        }
        println!("  Scripts:     {} .gd file(s)", info.script_count);
        println!("  Scenes:      {} .tscn file(s)", info.scene_count);
    }

    Ok(true)
}

fn parse_project_godot(content: &str) -> ProjectInfo {
    let mut name = String::new();
    let mut main_scene = None;
    let mut autoloads = Vec::new();

    let mut in_autoload_section = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Track sections
        if trimmed.starts_with('[') {
            in_autoload_section = trimmed == "[autoload]";
        }

        // Project name: config/name="MyProject"
        if let Some(val) = extract_value(trimmed, "config/name=") {
            name = val;
        }

        // Main scene: run/main_scene="res://scenes/main.tscn"
        if let Some(val) = extract_value(trimmed, "run/main_scene=") {
            main_scene = Some(val);
        }

        // Autoloads: Name="*res://path/to/script.gd"
        if in_autoload_section && trimmed.contains('=') && !trimmed.starts_with('[') {
            if let Some(eq_pos) = trimmed.find('=') {
                let autoload_name = trimmed[..eq_pos].trim().to_string();
                if !autoload_name.is_empty() {
                    autoloads.push(autoload_name);
                }
            }
        }
    }

    let script_count = count_files_recursive(Path::new("."), "gd");
    let scene_count = count_files_recursive(Path::new("."), "tscn");

    ProjectInfo {
        name,
        main_scene,
        autoloads,
        script_count,
        scene_count,
    }
}

/// Extract a string value from a line like `key="value"`.
fn extract_value(line: &str, key: &str) -> Option<String> {
    let rest = line.strip_prefix(key)?;
    // Remove surrounding quotes
    let val = rest.trim().trim_matches('"');
    Some(val.to_string())
}

/// Recursively count files with a given extension, skipping hidden dirs.
fn count_files_recursive(dir: &Path, extension: &str) -> usize {
    let mut count = 0;
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return 0,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        if name.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            count += count_files_recursive(&path, extension);
        } else if path.extension().is_some_and(|ext| ext == extension) {
            count += 1;
        }
    }

    count
}
