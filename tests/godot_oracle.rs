mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use wait_timeout::ChildExt;

use common::{cleanup, run_gdcli, setup_temp_project};

/// Find the Godot binary, mirroring src/godot_finder.rs logic.
/// Returns None if Godot is not available (tests will skip).
fn find_godot() -> Option<PathBuf> {
    // 1. GODOT_PATH env var
    if let Ok(env_path) = std::env::var("GODOT_PATH") {
        let p = PathBuf::from(&env_path);
        if p.is_file() {
            return Some(prefer_console_exe(&p));
        }
    }

    // 2. which("godot")
    if let Ok(p) = which::which("godot") {
        return Some(prefer_console_exe(&p));
    }

    // 3. Common Windows paths
    #[cfg(target_os = "windows")]
    {
        let candidates = [
            r"C:\Godot\godot.exe",
            r"C:\Godot\godot.console.exe",
            r"C:\Program Files\Godot\godot.exe",
            r"C:\Program Files\Godot\godot.console.exe",
        ];
        for candidate in &candidates {
            let p = PathBuf::from(candidate);
            if p.is_file() {
                return Some(prefer_console_exe(&p));
            }
        }

        if let Ok(appdata) = std::env::var("APPDATA") {
            let p = PathBuf::from(&appdata).join("Godot").join("godot.exe");
            if p.is_file() {
                return Some(prefer_console_exe(&p));
            }
        }
    }

    None
}

fn prefer_console_exe(path: &Path) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Some(ext) = path.extension() {
            if ext == "exe" {
                let stem = path.file_stem().unwrap_or_default().to_string_lossy();
                if !stem.ends_with(".console") {
                    let console_name = format!("{}.console.exe", stem);
                    let console_path = path.with_file_name(&console_name);
                    if console_path.is_file() {
                        return console_path;
                    }
                }
            }
        }
    }
    let _ = path;
    path.to_path_buf()
}

/// Run Godot in headless mode to validate a scene file.
/// Returns (success, stderr_output).
fn godot_loads_scene(godot: &Path, project_dir: &Path) -> (bool, String) {
    let child = Command::new(godot)
        .args(["--headless", "--quit"])
        .current_dir(project_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut child = match child {
        Ok(c) => c,
        Err(e) => return (false, format!("Failed to spawn Godot: {}", e)),
    };

    let timeout = Duration::from_secs(15);
    match child.wait_timeout(timeout) {
        Ok(Some(status)) => {
            let stdout = child.stdout.take().map(|mut s| {
                let mut buf = String::new();
                std::io::Read::read_to_string(&mut s, &mut buf).ok();
                buf
            }).unwrap_or_default();
            let stderr = child.stderr.take().map(|mut s| {
                let mut buf = String::new();
                std::io::Read::read_to_string(&mut s, &mut buf).ok();
                buf
            }).unwrap_or_default();
            let combined = format!("{}{}", stdout, stderr);
            let has_error = combined.lines().any(|line| {
                let upper = line.to_uppercase();
                upper.contains("ERROR") && !upper.contains("SCRIPT ERROR")
            });
            let success = status.success() && !has_error;
            (success, combined)
        }
        Ok(None) => {
            let _ = child.kill();
            (false, "Godot timed out after 15s".to_string())
        }
        Err(e) => (false, format!("Failed to wait on Godot: {}", e)),
    }
}

macro_rules! skip_without_godot {
    () => {
        match find_godot() {
            Some(g) => g,
            None => {
                eprintln!("SKIPPED: Godot binary not found — set GODOT_PATH or add godot to PATH");
                return;
            }
        }
    };
}

#[test]
fn test_godot_minimal_scene() {
    let godot = skip_without_godot!();
    let dir = setup_temp_project("oracle_minimal");
    let sp = dir.join("test.tscn");

    run_gdcli(&["scene", "create", sp.to_str().unwrap(), "--root-type", "Node2D"]);

    let (ok, output) = godot_loads_scene(&godot, &dir);
    assert!(ok, "Godot failed to load minimal scene:\n{}", output);

    cleanup(&dir);
}

#[test]
fn test_godot_scene_with_nodes() {
    let godot = skip_without_godot!();
    let dir = setup_temp_project("oracle_nodes");
    let sp = dir.join("test.tscn");
    let s = sp.to_str().unwrap();

    run_gdcli(&["scene", "create", s, "--root-type", "Node2D"]);
    run_gdcli(&["node", "add", s, "--node-type", "Label", "--name", "MyLabel"]);
    run_gdcli(&["node", "add", s, "--node-type", "Sprite2D", "--name", "MySprite"]);
    run_gdcli(&["node", "add", s, "--node-type", "Camera2D", "--name", "Cam"]);

    let (ok, output) = godot_loads_scene(&godot, &dir);
    assert!(ok, "Godot failed to load scene with nodes:\n{}", output);

    cleanup(&dir);
}

#[test]
fn test_godot_scene_with_properties() {
    let godot = skip_without_godot!();
    let dir = setup_temp_project("oracle_props");
    let sp = dir.join("test.tscn");
    let s = sp.to_str().unwrap();

    run_gdcli(&["scene", "create", s, "--root-type", "Node2D"]);
    run_gdcli(&["node", "add", s, "--node-type", "Label", "--name", "Lbl"]);
    run_gdcli(&["node", "add", s, "--node-type", "Sprite2D", "--name", "Spr"]);

    run_gdcli(&["--json", "scene", "edit", s, "--set", "Lbl::text=Hello"]);
    run_gdcli(&["--json", "scene", "edit", s, "--set", "Spr::position=Vector2(100, 200)"]);
    run_gdcli(&["--json", "scene", "edit", s, "--set", "Spr::visible=false"]);

    let (ok, output) = godot_loads_scene(&godot, &dir);
    assert!(ok, "Godot failed to load scene with properties:\n{}", output);

    cleanup(&dir);
}

#[test]
fn test_godot_scene_with_sub_resource() {
    let godot = skip_without_godot!();
    let dir = setup_temp_project("oracle_subres");
    let sp = dir.join("test.tscn");
    let s = sp.to_str().unwrap();

    run_gdcli(&["scene", "create", s, "--root-type", "Node2D"]);
    run_gdcli(&["node", "add", s, "--node-type", "CollisionShape2D", "--name", "Col"]);
    run_gdcli(&[
        "sub-resource", "add", s, "RectangleShape2D",
        "--props", "size=Vector2(32, 64)",
        "--wire-node", "Col",
        "--wire-property", "shape",
    ]);

    let (ok, output) = godot_loads_scene(&godot, &dir);
    assert!(ok, "Godot failed to load scene with sub-resource:\n{}", output);

    cleanup(&dir);
}

#[test]
fn test_godot_scene_with_connection() {
    let godot = skip_without_godot!();
    let dir = setup_temp_project("oracle_conn");
    let sp = dir.join("test.tscn");
    let s = sp.to_str().unwrap();

    run_gdcli(&["scene", "create", s, "--root-type", "Node2D"]);
    run_gdcli(&["node", "add", s, "--node-type", "Button", "--name", "Btn"]);
    run_gdcli(&["connection", "add", s, "pressed", "Btn", ".", "_on_pressed"]);

    let (ok, output) = godot_loads_scene(&godot, &dir);
    // Godot may warn about missing method but shouldn't hard-error
    assert!(ok, "Godot failed to load scene with connection:\n{}", output);

    cleanup(&dir);
}

#[test]
fn test_godot_scene_with_ext_resource() {
    let godot = skip_without_godot!();
    let dir = setup_temp_project("oracle_extres");
    let sp = dir.join("test.tscn");
    let s = sp.to_str().unwrap();

    run_gdcli(&["scene", "create", s, "--root-type", "Node2D"]);

    // Create a dummy SVG file
    fs::write(dir.join("icon.svg"), b"<svg></svg>").unwrap();

    run_gdcli(&["--json", "load-sprite", s, "MySprite", "res://icon.svg"]);

    let (ok, output) = godot_loads_scene(&godot, &dir);
    // Godot may warn about invalid SVG but shouldn't hard-error on load
    assert!(ok, "Godot failed to load scene with ext_resource:\n{}", output);

    cleanup(&dir);
}

#[test]
fn test_godot_complex_scene() {
    let godot = skip_without_godot!();
    let dir = setup_temp_project("oracle_complex");
    let sp = dir.join("test.tscn");
    let s = sp.to_str().unwrap();

    // Root Node2D
    run_gdcli(&["scene", "create", s, "--root-type", "Node2D"]);

    // Label with text property
    run_gdcli(&["node", "add", s, "--node-type", "Label", "--name", "Title"]);
    run_gdcli(&["--json", "scene", "edit", s, "--set", "Title::text=Game Title"]);

    // CharacterBody2D with CollisionShape2D + sub-resource
    run_gdcli(&["node", "add", s, "--node-type", "CharacterBody2D", "--name", "Player"]);
    run_gdcli(&["node", "add", s, "--node-type", "CollisionShape2D", "--name", "PlayerCol", "--parent", "Player"]);
    run_gdcli(&[
        "sub-resource", "add", s, "RectangleShape2D",
        "--props", "size=Vector2(16, 32)",
        "--wire-node", "PlayerCol",
        "--wire-property", "shape",
    ]);

    // Button with signal connection
    run_gdcli(&["node", "add", s, "--node-type", "Button", "--name", "StartBtn"]);
    run_gdcli(&["connection", "add", s, "pressed", "StartBtn", ".", "_on_start"]);

    let (ok, output) = godot_loads_scene(&godot, &dir);
    assert!(ok, "Godot failed to load complex scene:\n{}", output);

    cleanup(&dir);
}
