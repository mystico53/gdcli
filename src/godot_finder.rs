use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

use crate::runner;

pub struct GodotInfo {
    pub path: PathBuf,
    pub version: String,
}

/// Find the Godot binary and probe its capabilities.
pub fn find_and_probe() -> Result<GodotInfo> {
    let path = find_binary()?;
    let version = probe_version(&path)?;

    Ok(GodotInfo { path, version })
}

/// Search for the Godot binary in order of priority:
/// 1. GODOT_PATH environment variable
/// 2. `godot` on PATH (via `which`)
/// 3. Common Windows install locations
fn find_binary() -> Result<PathBuf> {
    // 1. GODOT_PATH env var
    if let Ok(env_path) = std::env::var("GODOT_PATH") {
        let p = PathBuf::from(&env_path);
        if p.is_file() {
            return Ok(prefer_console_exe(&p));
        }
        bail!(
            "GODOT_PATH is set to '{}' but the file does not exist.\n\
             Please check your GODOT_PATH environment variable.",
            env_path
        );
    }

    // 2. which("godot")
    if let Ok(p) = which::which("godot") {
        return Ok(prefer_console_exe(&p));
    }

    // 3. Common Windows paths (including versioned executables)
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
                return Ok(prefer_console_exe(&p));
            }
        }

        // Scan common directories for versioned Godot executables
        // (e.g. C:\Godot\Godot_v4.6.1-stable_win64.exe)
        let scan_dirs = [
            r"C:\Godot",
            r"C:\Program Files\Godot",
        ];
        for dir in &scan_dirs {
            if let Some(found) = find_versioned_godot(Path::new(dir)) {
                return Ok(prefer_console_exe(&found));
            }
        }

        // Check %APPDATA%\Godot
        if let Ok(appdata) = std::env::var("APPDATA") {
            let appdata_godot = PathBuf::from(&appdata).join("Godot");
            let p = appdata_godot.join("godot.exe");
            if p.is_file() {
                return Ok(prefer_console_exe(&p));
            }
            if let Some(found) = find_versioned_godot(&appdata_godot) {
                return Ok(prefer_console_exe(&found));
            }
        }

        // Check %LOCALAPPDATA%\Godot
        if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
            let local_godot = PathBuf::from(&localappdata).join("Godot");
            if let Some(found) = find_versioned_godot(&local_godot) {
                return Ok(prefer_console_exe(&found));
            }
        }

        // Check user's home directories
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            let home_dirs = [
                PathBuf::from(&userprofile).join("Godot"),
                PathBuf::from(&userprofile).join("scoop").join("apps").join("godot"),
            ];
            for dir in &home_dirs {
                if let Some(found) = find_versioned_godot(dir) {
                    return Ok(prefer_console_exe(&found));
                }
            }
        }
    }

    // 3. Common macOS/Linux paths
    #[cfg(not(target_os = "windows"))]
    {
        let candidates = [
            "/usr/local/bin/godot",
            "/usr/bin/godot",
            "/opt/godot/godot",
        ];
        for candidate in &candidates {
            let p = PathBuf::from(candidate);
            if p.is_file() {
                return Ok(p);
            }
        }

        // macOS: check Applications
        #[cfg(target_os = "macos")]
        {
            let app_path = PathBuf::from("/Applications/Godot.app/Contents/MacOS/Godot");
            if app_path.is_file() {
                return Ok(app_path);
            }
        }
    }

    bail!(
        "Could not find a Godot binary.\n\n\
         Set the GODOT_PATH environment variable to point to your Godot binary:\n  \
         set GODOT_PATH=C:\\path\\to\\godot.exe\n\n\
         Or add Godot to your PATH."
    );
}

/// Scan a directory for versioned Godot executables (e.g. Godot_v4.6.1-stable_win64.exe).
/// Returns the newest version found, preferring console executables.
#[cfg(target_os = "windows")]
fn find_versioned_godot(dir: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    let mut best: Option<PathBuf> = None;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = path.file_name()?.to_string_lossy().to_lowercase();
        // Match patterns like "godot_v4.6.1-stable_win64.exe" or "Godot_v4.4-stable_win64.exe"
        if name.starts_with("godot") && name.ends_with(".exe") && !name.contains("console") {
            // Prefer newer versions (lexicographic comparison works for Godot versioning)
            if best.as_ref().map_or(true, |b| {
                path.file_name().unwrap().to_string_lossy() > b.file_name().unwrap().to_string_lossy()
            }) {
                best = Some(path);
            }
        }
    }

    best
}

/// On Windows, the GUI `.exe` doesn't write to stdout. If a `.console.exe`
/// sibling exists, prefer that instead so we can capture output.
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
    let _ = path; // suppress unused warning on non-windows
    path.to_path_buf()
}

/// Run `godot --version` and extract the version string.
fn probe_version(godot_path: &Path) -> Result<String> {
    let result =
        runner::run_raw(godot_path, &["--version"], 10).context("failed to probe Godot version")?;

    let version = result
        .stdout
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .to_string();

    if version.is_empty() {
        bail!(
            "Godot at '{}' returned no version output.\n\
             Is this a valid Godot binary?",
            godot_path.display()
        );
    }

    Ok(version)
}
