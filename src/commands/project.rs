use anyhow::{bail, Result};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

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

// --- project init ---

#[derive(Serialize)]
pub struct ProjectInitReport {
    pub path: String,
    pub name: String,
    pub godot_version: String,
    pub renderer: String,
}

pub fn run_init(
    dir: Option<&str>,
    name: Option<&str>,
    godot_version: Option<&str>,
    renderer: Option<&str>,
    force: bool,
    json_mode: bool,
) -> Result<bool> {
    let project_dir = match dir {
        Some(d) => {
            let p = PathBuf::from(d);
            if !p.exists() {
                fs::create_dir_all(&p)?;
            }
            p.canonicalize()?
        }
        None => std::env::current_dir()?,
    };

    let project_file = project_dir.join("project.godot");
    if project_file.is_file() && !force {
        bail!(
            "project.godot already exists at {}\nUse --force to overwrite.",
            project_file.display()
        );
    }

    // Determine project name
    let resolved_name = match name {
        Some(n) => n.to_string(),
        None => project_dir
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "MyProject".to_string()),
    };

    // Determine Godot version (major.minor)
    let resolved_version = match godot_version {
        Some(v) => extract_major_minor(v),
        None => {
            match crate::godot_finder::find_and_probe() {
                Ok(info) => extract_major_minor(&info.version),
                Err(_) => bail!(
                    "Could not detect Godot version.\n\
                     Provide --godot-version (e.g. --godot-version 4.6)"
                ),
            }
        }
    };

    // Determine renderer
    let (renderer_label, renderer_method) = match renderer.unwrap_or("forward_plus") {
        "mobile" => ("Mobile", "mobile"),
        "gl_compatibility" | "compatibility" => ("GL Compatibility", "gl_compatibility"),
        _ => ("Forward Plus", "forward_plus"),
    };

    // Generate project.godot content matching Godot's own format
    let content = format!(
        "\
; Engine configuration file.
; It's best edited using the editor UI and not directly,
; since the parameters that go here are not all obvious.
;
; Format:
;   [section] ; section goes between []
;   param=value ; assign values to parameters

config_version=5

[application]

config/name=\"{name}\"
config/features=PackedStringArray(\"{version}\", \"{renderer_label}\")

[rendering]

renderer/rendering_method=\"{renderer_method}\"
",
        name = resolved_name,
        version = resolved_version,
        renderer_label = renderer_label,
        renderer_method = renderer_method,
    );

    fs::write(&project_file, &content)?;

    if json_mode {
        let report = ProjectInitReport {
            path: project_file.display().to_string(),
            name: resolved_name.clone(),
            godot_version: resolved_version.clone(),
            renderer: renderer_label.to_string(),
        };
        let envelope = output::JsonEnvelope {
            ok: true,
            command: "project init".into(),
            data: Some(report),
            error: None,
        };
        output::emit_json(&envelope);
    } else {
        println!(
            "  \u{2713} Created project.godot at {}",
            project_file.display()
        );
        println!("    Name:     {}", resolved_name);
        println!("    Version:  {}", resolved_version);
        println!("    Renderer: {}", renderer_label);
    }

    Ok(true)
}

/// Extract major.minor from a version string like "4.6.1.stable" → "4.6"
fn extract_major_minor(version: &str) -> String {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() >= 2 {
        format!("{}.{}", parts[0], parts[1])
    } else {
        version.to_string()
    }
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
