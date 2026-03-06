use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

/// Strip the `res://` prefix from a path string if present.
/// This allows MCP callers to use Godot-style resource paths (e.g. `res://main.tscn`)
/// which get converted to relative filesystem paths (e.g. `main.tscn`).
pub fn strip_res_prefix(path: &str) -> &str {
    path.strip_prefix("res://").unwrap_or(path)
}

/// Walk up from `start` looking for a directory containing `project.godot`.
/// Returns the directory path (not the file itself).
pub fn find_project_root(start: &Path) -> Option<PathBuf> {
    let mut dir = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start.to_path_buf()
    };

    loop {
        if dir.join("project.godot").is_file() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Find the Godot project root and `std::env::set_current_dir` to it.
///
/// Strategy:
/// 1. If `hint` is provided (e.g. a file path from a command arg), walk up from it.
/// 2. Otherwise walk up from the current working directory.
/// 3. If neither finds `project.godot`, bail with a clear error.
pub fn ensure_project_context(hint: Option<&Path>) -> Result<PathBuf> {
    // Try hint path first
    if let Some(hint_path) = hint {
        let abs = if hint_path.is_absolute() {
            hint_path.to_path_buf()
        } else {
            std::env::current_dir()?.join(hint_path)
        };

        if let Some(root) = find_project_root(&abs) {
            std::env::set_current_dir(&root)?;
            return Ok(root);
        }
    }

    // Fall back to CWD walk-up
    let cwd = std::env::current_dir()?;
    if let Some(root) = find_project_root(&cwd) {
        std::env::set_current_dir(&root)?;
        return Ok(root);
    }

    bail!(
        "project.godot not found.\n\
         Run this command from inside a Godot project, or pass a file path within one."
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_find_project_root_direct() {
        let dir = std::env::temp_dir().join("gdcli_test_projutil");
        let _ = fs::create_dir_all(&dir);
        fs::write(dir.join("project.godot"), "").unwrap();

        assert_eq!(find_project_root(&dir), Some(dir.clone()));

        let _ = fs::remove_file(dir.join("project.godot"));
        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn test_find_project_root_from_subdir() {
        let dir = std::env::temp_dir().join("gdcli_test_projutil2");
        let sub = dir.join("scenes").join("enemies");
        let _ = fs::create_dir_all(&sub);
        fs::write(dir.join("project.godot"), "").unwrap();

        assert_eq!(find_project_root(&sub), Some(dir.clone()));

        let _ = fs::remove_file(dir.join("project.godot"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_find_project_root_from_file() {
        let dir = std::env::temp_dir().join("gdcli_test_projutil3");
        let sub = dir.join("scripts");
        let _ = fs::create_dir_all(&sub);
        fs::write(dir.join("project.godot"), "").unwrap();
        let file = sub.join("player.gd");
        fs::write(&file, "extends Node").unwrap();

        assert_eq!(find_project_root(&file), Some(dir.clone()));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_find_project_root_not_found() {
        // A path that definitely won't have project.godot
        let dir = std::env::temp_dir().join("gdcli_test_no_project_xyz");
        let _ = fs::create_dir_all(&dir);

        assert_eq!(find_project_root(&dir), None);

        let _ = fs::remove_dir(&dir);
    }

    #[test]
    fn test_strip_res_prefix() {
        assert_eq!(strip_res_prefix("res://main.tscn"), "main.tscn");
        assert_eq!(
            strip_res_prefix("res://scenes/player.tscn"),
            "scenes/player.tscn"
        );
        assert_eq!(strip_res_prefix("main.tscn"), "main.tscn");
        assert_eq!(strip_res_prefix(""), "");
        assert_eq!(strip_res_prefix("res://"), "");
        // Only strip the prefix, not a partial match
        assert_eq!(strip_res_prefix("res:/main.tscn"), "res:/main.tscn");
    }
}
