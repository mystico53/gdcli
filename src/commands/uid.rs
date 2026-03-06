use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::output;
use crate::project_util;
use crate::scene_parser;

#[derive(Serialize)]
pub struct UidFixReport {
    pub fixes: Vec<UidFix>,
    pub fix_count: usize,
    pub dry_run: bool,
}

#[derive(Serialize)]
pub struct UidFix {
    pub file: String,
    pub resource_id: String,
    pub uid: String,
    pub old_path: String,
    pub new_path: String,
}

pub fn run_fix(dry_run: bool, json_mode: bool) -> Result<bool> {
    project_util::ensure_project_context(None)?;

    // Build UID → path map from the .godot/uid_cache.bin or by scanning .uid files
    let uid_map = build_uid_map()?;

    if uid_map.is_empty() {
        if json_mode {
            let report = UidFixReport {
                fixes: Vec::new(),
                fix_count: 0,
                dry_run,
            };
            let envelope = output::JsonEnvelope {
                ok: true,
                command: "uid fix".into(),
                data: Some(report),
                error: None,
            };
            output::emit_json(&envelope);
        } else {
            println!("  No UID mappings found. Nothing to fix.");
        }
        return Ok(true);
    }

    // Scan all .tscn and .tres files for ext_resource entries with UIDs
    let mut fixes = Vec::new();
    let scene_files = find_resource_files(Path::new("."));

    for file_path in &scene_files {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let display_path = file_path
            .strip_prefix(".")
            .unwrap_or(file_path)
            .display()
            .to_string()
            .replace('\\', "/");

        // Parse ext_resources
        let parsed = match scene_parser::parse_scene_text(&content) {
            Ok(p) => p,
            Err(_) => continue,
        };

        for ext_res in &parsed.ext_resources {
            if let Some(ref uid) = ext_res.uid {
                if let Some(correct_path) = uid_map.get(uid.as_str()) {
                    let current_path = &ext_res.path;
                    if current_path != correct_path {
                        fixes.push(UidFix {
                            file: display_path.clone(),
                            resource_id: ext_res.id.clone(),
                            uid: uid.clone(),
                            old_path: current_path.clone(),
                            new_path: correct_path.clone(),
                        });
                    }
                }
            }
        }
    }

    let fix_count = fixes.len();
    let clean = fix_count == 0;

    // Apply fixes if not dry-run
    if !dry_run && !clean {
        apply_fixes(&fixes)?;
    }

    if json_mode {
        let report = UidFixReport {
            fixes,
            fix_count,
            dry_run,
        };
        let envelope = output::JsonEnvelope {
            ok: true,
            command: "uid fix".into(),
            data: Some(report),
            error: None,
        };
        output::emit_json(&envelope);
    } else if clean {
        println!("  \u{2713} No stale UID references found");
    } else {
        let action = if dry_run { "would fix" } else { "fixed" };
        output::print_header(&format!("{} {} stale UID reference(s)", action, fix_count));
        for fix in &fixes {
            println!(
                "  {} [{}] {} → {}",
                fix.file, fix.uid, fix.old_path, fix.new_path
            );
        }
    }

    Ok(true)
}

/// Apply path fixes to the actual files.
fn apply_fixes(fixes: &[UidFix]) -> Result<()> {
    // Group fixes by file
    let mut by_file: HashMap<&str, Vec<&UidFix>> = HashMap::new();
    for fix in fixes {
        by_file.entry(fix.file.as_str()).or_default().push(fix);
    }

    for (rel_path, file_fixes) in &by_file {
        // Convert display path back to filesystem path
        let fs_path = Path::new(".").join(rel_path.replace('/', std::path::MAIN_SEPARATOR_STR));
        let mut content = fs::read_to_string(&fs_path)?;

        for fix in file_fixes {
            // Replace the old path with the new path in ext_resource lines
            let old_pattern = format!("path=\"{}\"", fix.old_path);
            let new_pattern = format!("path=\"{}\"", fix.new_path);
            content = content.replace(&old_pattern, &new_pattern);
        }

        fs::write(&fs_path, &content)?;
    }

    Ok(())
}

/// Build a UID → res:// path map by scanning .uid files in the project.
/// Godot 4.4+ creates .uid files alongside resources.
fn build_uid_map() -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    scan_uid_files(Path::new("."), &mut map);
    Ok(map)
}

fn scan_uid_files(dir: &Path, map: &mut HashMap<String, String>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if name_str.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            scan_uid_files(&path, map);
        } else if name_str.ends_with(".uid") {
            // .uid file contains the UID, the resource path is derived from the filename
            // e.g. main.gd.uid contains the UID for res://main.gd
            if let Ok(uid_content) = fs::read_to_string(&path) {
                let uid = uid_content.trim().to_string();
                if uid.starts_with("uid://") {
                    // Derive the resource path from the .uid file path
                    let resource_path = path.with_extension(""); // strip .uid
                    let rel = resource_path
                        .strip_prefix(".")
                        .unwrap_or(&resource_path)
                        .display()
                        .to_string()
                        .replace('\\', "/");
                    // strip_prefix(".") on Windows gives "path\to\file" (no leading sep)
                    let rel = rel.trim_start_matches('/');
                    let res_path = format!("res://{}", rel);
                    map.insert(uid, res_path);
                }
            }
        }
    }
}

/// Find all .tscn and .tres files recursively.
fn find_resource_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    find_resource_files_recursive(dir, &mut files);
    files.sort();
    files
}

fn find_resource_files_recursive(dir: &Path, results: &mut Vec<std::path::PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if name_str.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            find_resource_files_recursive(&path, results);
        } else if path
            .extension()
            .is_some_and(|ext| ext == "tscn" || ext == "tres")
        {
            results.push(path);
        }
    }
}
