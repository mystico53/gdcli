use anyhow::{bail, Result};
use serde::Serialize;
use std::path::Path;

use crate::errors::{self, GodotError};
use crate::godot_finder::GodotInfo;
use crate::output;
use crate::runner;

#[derive(Serialize)]
pub struct RunReport {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub errors: Vec<GodotError>,
    pub error_count: usize,
    pub timed_out: bool,
    pub duration_ms: u64,
}

pub fn run_project(
    godot_info: &GodotInfo,
    timeout: u64,
    scene: Option<&str>,
    json_mode: bool,
) -> Result<bool> {
    if !Path::new("project.godot").is_file() {
        bail!(
            "project.godot not found in current directory.\n\
             Run this command from your Godot project root."
        );
    }

    let mut args: Vec<&str> = Vec::new();

    if let Some(scene_path) = scene {
        args.push("--scene");
        args.push(scene_path);
    }

    let result = runner::run(&godot_info.path, &args, timeout)?;

    let all_output = format!("{}\n{}", result.stdout, result.stderr);
    let errors = errors::parse_script_errors(&all_output);
    let error_count = errors.len();
    let ok = result.exit_code == 0 && error_count == 0 && !result.timed_out;

    if json_mode {
        let report = RunReport {
            exit_code: result.exit_code,
            stdout: result.stdout,
            stderr: result.stderr,
            errors,
            error_count,
            timed_out: result.timed_out,
            duration_ms: result.duration_ms,
        };
        let envelope = output::JsonEnvelope {
            ok,
            command: "run".into(),
            data: Some(report),
            error: if ok {
                None
            } else if result.timed_out {
                Some(format!("Process timed out after {}s", timeout))
            } else if error_count > 0 {
                Some(format!("{} runtime error(s)", error_count))
            } else {
                Some(format!("Process exited with code {}", result.exit_code))
            },
        };
        output::emit_json(&envelope);
    } else {
        // Print stdout as-is (the actual program output)
        if !result.stdout.trim().is_empty() {
            println!("{}", result.stdout.trim());
        }

        if result.timed_out {
            println!();
            output::print_error(&format!("Process timed out after {}s", timeout));
        }

        if error_count > 0 {
            println!();
            output::print_header("Runtime errors:");
            for err in &errors {
                println!("  \u{2717} {}", errors::format_error_tty(err));
            }
            println!();
            output::print_error(&format!("{} error(s)", error_count));
        }

        if !result.timed_out && error_count == 0 && result.exit_code != 0 {
            output::print_error(&format!("Process exited with code {}", result.exit_code));
        }
    }

    Ok(ok)
}
