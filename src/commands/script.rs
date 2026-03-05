use anyhow::{bail, Result};
use serde::Serialize;
use std::path::Path;

use crate::errors::{self, GodotError};
use crate::godot_finder::GodotInfo;
use crate::output;
use crate::runner;

#[derive(Serialize)]
pub struct LintReport {
    pub errors: Vec<GodotError>,
    pub error_count: usize,
    pub file: Option<String>,
}

pub fn run_lint(godot_info: &GodotInfo, file: Option<&str>, json_mode: bool) -> Result<bool> {
    if !godot_info.structured_errors_supported {
        bail!(
            "Your Godot build does not support --structured-errors.\n\
             gdcli requires a patched Godot build.\n\
             Set GODOT_PATH to point to your patched binary."
        );
    }

    let (result, use_script_errors) = if let Some(file_path) = file {
        (lint_single_file(godot_info, file_path)?, true)
    } else {
        (lint_project(godot_info)?, false)
    };

    let all_output = format!("{}\n{}", result.stdout, result.stderr);

    // Single-file lint uses --check-only WITHOUT --structured-errors
    // (because --structured-errors implies -d which prevents --check-only from exiting).
    // Project-wide lint uses --structured-errors --quit normally.
    let errors = if use_script_errors {
        errors::parse_script_errors(&all_output)
    } else {
        errors::parse_errors(&all_output)
    };

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
/// Uses `--headless` but NOT `--structured-errors` because the latter implies `-d`
/// which prevents `--check-only` from exiting. We parse SCRIPT ERROR blocks instead.
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
/// Uses `--structured-errors` for clean error parsing.
fn lint_project(godot_info: &GodotInfo) -> Result<runner::RunResult> {
    if !Path::new("project.godot").is_file() {
        bail!(
            "project.godot not found in current directory.\n\
             Run this command from your Godot project root."
        );
    }

    runner::run(&godot_info.path, &["--quit"], 60)
}
