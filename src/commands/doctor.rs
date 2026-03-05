use anyhow::Result;
use serde::Serialize;
use std::fs;
use std::path::Path;

use crate::godot_finder::GodotInfo;
use crate::output;

#[derive(Serialize)]
pub struct DoctorReport {
    pub checks: Vec<CheckResult>,
    pub all_passed: bool,
}

#[derive(Serialize)]
pub struct CheckResult {
    pub name: String,
    pub passed: bool,
    pub message: String,
}

pub fn run(godot_info: &GodotInfo, json_mode: bool) -> Result<bool> {
    let mut checks = Vec::new();

    // Check 1: Godot binary found
    checks.push(CheckResult {
        name: "godot_binary".into(),
        passed: true,
        message: format!(
            "Godot {} found at {}",
            godot_info.version,
            godot_info.path.display()
        ),
    });

    // Check 2: --structured-errors supported
    checks.push(CheckResult {
        name: "structured_errors".into(),
        passed: godot_info.structured_errors_supported,
        message: if godot_info.structured_errors_supported {
            "--structured-errors supported".into()
        } else {
            "--structured-errors NOT supported — gdcli requires a patched Godot build".into()
        },
    });

    // Check 3: project.godot exists in CWD
    let project_exists = Path::new("project.godot").is_file();
    checks.push(CheckResult {
        name: "project_file".into(),
        passed: project_exists,
        message: if project_exists {
            "project.godot found".into()
        } else {
            "project.godot not found in current directory".into()
        },
    });

    // Check 4: Count .gd files
    let gd_count = count_gd_files(Path::new("."));
    checks.push(CheckResult {
        name: "gd_files".into(),
        passed: true,
        message: format!("{} .gd file(s) found", gd_count),
    });

    let all_passed = checks.iter().all(|c| c.passed);

    if json_mode {
        let report = DoctorReport { checks, all_passed };
        let envelope = output::JsonEnvelope {
            ok: all_passed,
            command: "doctor".into(),
            data: Some(report),
            error: if all_passed {
                None
            } else {
                Some("one or more checks failed".into())
            },
        };
        output::emit_json(&envelope);
    } else {
        output::print_header("gdcli doctor");
        for check in &checks {
            output::print_check(check.passed, &check.message);
        }
        if !all_passed {
            println!();
            output::print_error("one or more checks failed");
        }
    }

    Ok(all_passed)
}

/// Recursively count `.gd` files, skipping `.godot/` and hidden directories.
fn count_gd_files(dir: &Path) -> usize {
    let mut count = 0;

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return 0,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        // Skip hidden dirs and .godot/
        if name.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            count += count_gd_files(&path);
        } else if path.extension().is_some_and(|ext| ext == "gd") {
            count += 1;
        }
    }

    count
}
