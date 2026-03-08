use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::output;

#[derive(Serialize)]
struct SetupReport {
    target: String,
    action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    config: Value,
}

fn mcp_server_config() -> Value {
    if cfg!(target_os = "windows") {
        json!({
            "command": "npx.cmd",
            "args": ["-y", "gdcli-godot", "mcp"]
        })
    } else {
        json!({
            "command": "npx",
            "args": ["-y", "gdcli-godot", "mcp"]
        })
    }
}

pub fn run_claude_code(json_mode: bool) -> Result<bool> {
    let mut args: Vec<&str> = vec!["mcp", "add", "--transport", "stdio", "gdcli", "--"];

    if cfg!(target_os = "windows") {
        args.extend(["npx.cmd", "-y", "gdcli-godot", "mcp"]);
    } else {
        args.extend(["npx", "-y", "gdcli-godot", "mcp"]);
    }

    let result = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/c", "claude"])
            .args(&args)
            .output()
    } else {
        Command::new("claude").args(&args).output()
    };

    match result {
        Ok(output) => {
            if output.status.success() {
                let report = SetupReport {
                    target: "claude-code".into(),
                    action: "configured".into(),
                    path: None,
                    config: mcp_server_config(),
                };

                if json_mode {
                    output::emit_json(&output::JsonEnvelope {
                        ok: true,
                        command: "setup".into(),
                        data: Some(&report),
                        error: None,
                    });
                } else {
                    output::print_check(true, "gdcli MCP server added to Claude Code");
                }
                Ok(true)
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let msg = format!("claude mcp add failed: {}", stderr.trim());
                if json_mode {
                    output::emit_json(&output::JsonEnvelope::<()> {
                        ok: false,
                        command: "setup".into(),
                        data: None,
                        error: Some(msg),
                    });
                } else {
                    output::print_error(&msg);
                }
                Ok(false)
            }
        }
        Err(_) => {
            let msg = "Could not find `claude` CLI. Install it from https://docs.anthropic.com/en/docs/claude-code or use `gdcli setup json` to get the config manually.";
            if json_mode {
                output::emit_json(&output::JsonEnvelope::<()> {
                    ok: false,
                    command: "setup".into(),
                    data: None,
                    error: Some(msg.into()),
                });
            } else {
                output::print_error(msg);
            }
            Ok(false)
        }
    }
}

pub fn run_cursor(json_mode: bool) -> Result<bool> {
    write_mcp_json_config(Path::new(".cursor/mcp.json"), "cursor", json_mode)
}

pub fn run_vscode(json_mode: bool) -> Result<bool> {
    write_mcp_json_config(Path::new(".vscode/mcp.json"), "vscode", json_mode)
}

fn write_mcp_json_config(path: &Path, target: &str, json_mode: bool) -> Result<bool> {
    let mut root = if path.exists() {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        serde_json::from_str::<Value>(&content).with_context(|| {
            format!(
                "{} contains comments or non-standard JSON. Remove comments and try again, or use `gdcli setup json` for manual insertion.",
                path.display()
            )
        })?
    } else {
        json!({})
    };

    let action = if path.exists() { "updated" } else { "created" };

    // Ensure mcpServers object exists
    if root.get("mcpServers").is_none() {
        root.as_object_mut()
            .unwrap()
            .insert("mcpServers".into(), json!({}));
    }

    // Insert/replace the gdcli entry
    root["mcpServers"]
        .as_object_mut()
        .unwrap()
        .insert("gdcli".into(), mcp_server_config());

    // Write the file
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }
    let pretty = serde_json::to_string_pretty(&root)?;
    fs::write(path, format!("{pretty}\n"))
        .with_context(|| format!("Failed to write {}", path.display()))?;

    let report = SetupReport {
        target: target.into(),
        action: action.into(),
        path: Some(path.display().to_string()),
        config: mcp_server_config(),
    };

    if json_mode {
        output::emit_json(&output::JsonEnvelope {
            ok: true,
            command: "setup".into(),
            data: Some(&report),
            error: None,
        });
    } else {
        output::print_check(
            true,
            &format!("{} {}", action.chars().next().unwrap().to_uppercase().to_string() + &action[1..], path.display()),
        );
    }

    Ok(true)
}

pub fn run_json(json_mode: bool) -> Result<bool> {
    let config = json!({
        "mcpServers": {
            "gdcli": mcp_server_config()
        }
    });

    if json_mode {
        let report = SetupReport {
            target: "json".into(),
            action: "printed".into(),
            path: None,
            config: config.clone(),
        };
        output::emit_json(&output::JsonEnvelope {
            ok: true,
            command: "setup".into(),
            data: Some(&report),
            error: None,
        });
    } else {
        println!("Add this to your MCP config file:\n");
        println!("{}", serde_json::to_string_pretty(&config)?);
    }

    Ok(true)
}
