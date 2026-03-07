#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub fn gdcli_bin() -> PathBuf {
    let mut path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    if cfg!(windows) {
        path.push("gdcli.exe");
    } else {
        path.push("gdcli");
    }
    path
}

pub fn setup_temp_project(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("gdcli_cli_test_{}", name));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("project.godot"),
        "[gd_resource type=\"ProjectSettings\" format=3]\n\n[resource]\n",
    )
    .unwrap();
    dir
}

pub fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

pub fn run_gdcli(args: &[&str]) -> Output {
    Command::new(gdcli_bin())
        .args(args)
        .output()
        .expect("Failed to run gdcli")
}

pub fn run_gdcli_in(dir: &Path, args: &[&str]) -> Output {
    Command::new(gdcli_bin())
        .args(args)
        .current_dir(dir)
        .output()
        .expect("Failed to run gdcli")
}

pub fn assert_ok_json(output: &Output) -> serde_json::Value {
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "gdcli failed: {}{}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("Invalid JSON: {}\nOutput: {}", e, stdout));
    assert_eq!(json["ok"], true, "Expected ok=true, got: {}", json);
    json
}
