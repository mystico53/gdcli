use anyhow::{bail, Result};
use serde::Serialize;
use std::fs;
use std::path::Path;

use crate::errors::{self, GodotError};
use crate::godot_finder::GodotInfo;
use crate::output;
use crate::runner;
use crate::scene_parser;

// --- script create ---

#[derive(Serialize)]
pub struct ScriptCreateReport {
    pub path: String,
    pub extends: String,
    pub methods: Vec<String>,
}

/// Known method signatures for common Godot lifecycle methods.
fn method_signature(name: &str) -> &str {
    match name {
        "_ready" => "func _ready() -> void:\n\tpass",
        "_process" => "func _process(delta: float) -> void:\n\tpass",
        "_physics_process" => "func _physics_process(delta: float) -> void:\n\tpass",
        "_input" => "func _input(event: InputEvent) -> void:\n\tpass",
        "_unhandled_input" => "func _unhandled_input(event: InputEvent) -> void:\n\tpass",
        "_enter_tree" => "func _enter_tree() -> void:\n\tpass",
        "_exit_tree" => "func _exit_tree() -> void:\n\tpass",
        _ => "",
    }
}

pub fn run_create(
    script_path: &str,
    extends: &str,
    methods: &[String],
    force: bool,
    json_mode: bool,
) -> Result<bool> {
    let path = Path::new(script_path);

    if path.is_file() && !force {
        bail!(
            "File already exists: {}\nUse --force to overwrite.",
            script_path
        );
    }

    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let mut content = format!("extends {}\n", extends);

    for method_name in methods {
        let sig = method_signature(method_name);
        if sig.is_empty() {
            // Unknown method — generate a basic stub
            content.push_str(&format!("\n\nfunc {}() -> void:\n\tpass", method_name));
        } else {
            content.push_str(&format!("\n\n{}", sig));
        }
    }

    content.push('\n');

    scene_parser::atomic_write(path, &content)?;

    if json_mode {
        let report = ScriptCreateReport {
            path: script_path.to_string(),
            extends: extends.to_string(),
            methods: methods.to_vec(),
        };
        let envelope = output::JsonEnvelope {
            ok: true,
            command: "script create".into(),
            data: Some(report),
            error: None,
        };
        output::emit_json(&envelope);
    } else {
        println!("  \u{2713} Created {} (extends {})", script_path, extends);
        if !methods.is_empty() {
            println!("    methods: {}", methods.join(", "));
        }
    }

    Ok(true)
}

// --- script lint ---

#[derive(Serialize)]
pub struct LintReport {
    pub errors: Vec<GodotError>,
    pub error_count: usize,
    pub file: Option<String>,
}

pub fn run_lint(godot_info: &GodotInfo, file: Option<&str>, json_mode: bool) -> Result<bool> {
    let result = if let Some(file_path) = file {
        lint_single_file(godot_info, file_path)?
    } else {
        lint_project(godot_info)?
    };

    let all_output = format!("{}\n{}", result.stdout, result.stderr);
    let errors = errors::parse_script_errors(&all_output);

    let error_count = errors.len();
    let clean = error_count == 0;

    if json_mode {
        let report = LintReport {
            errors,
            error_count,
            file: file.map(String::from),
        };
        let envelope = output::JsonEnvelope {
            ok: clean,
            command: "script lint".into(),
            data: Some(report),
            error: if clean {
                None
            } else {
                Some(format!("{} error(s) found", error_count))
            },
        };
        output::emit_json(&envelope);
    } else if clean {
        if let Some(f) = file {
            println!("  \u{2713} {} — clean", f);
        } else {
            println!("  \u{2713} No script errors found");
        }
    } else {
        output::print_header("Script errors:");
        for err in &errors {
            println!("  \u{2717} {}", errors::format_error_tty(err));
        }
        println!();
        output::print_error(&format!("{} error(s) found", error_count));
    }

    Ok(clean)
}

/// Lint a single file using `--check-only -s <file>`.
fn lint_single_file(godot_info: &GodotInfo, file_path: &str) -> Result<runner::RunResult> {
    if !Path::new(file_path).is_file() {
        bail!("File not found: {}", file_path);
    }

    if !Path::new("project.godot").is_file() {
        bail!(
            "project.godot not found in current directory.\n\
             Run this command from your Godot project root."
        );
    }

    runner::run_raw(
        &godot_info.path,
        &["--headless", "-s", file_path, "--check-only"],
        30,
    )
}

/// Lint the whole project by loading it with `--quit`.
fn lint_project(godot_info: &GodotInfo) -> Result<runner::RunResult> {
    if !Path::new("project.godot").is_file() {
        bail!(
            "project.godot not found in current directory.\n\
             Run this command from your Godot project root."
        );
    }

    runner::run(&godot_info.path, &["--quit"], 60)
}
