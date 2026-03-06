use serde_json::Value;

use crate::commands;
use crate::godot_finder;
use crate::output;
use crate::scene_parser;

pub struct ToolResult {
    pub is_error: bool,
    pub text: String,
}

pub fn call_tool(name: &str, args: &Value) -> ToolResult {
    output::begin_capture();

    let result = match name {
        "doctor" => dispatch_doctor(),
        "project_info" => dispatch_project_info(),
        "scene_list" => dispatch_scene_list(),
        "scene_validate" => dispatch_scene_validate(args),
        "scene_create" => dispatch_scene_create(args),
        "scene_edit" => dispatch_scene_edit(args),
        "node_add" => dispatch_node_add(args),
        "node_remove" => dispatch_node_remove(args),
        "uid_fix" => dispatch_uid_fix(args),
        "script_create" => dispatch_script_create(args),
        "script_lint" => dispatch_script_lint(args),
        "run" => dispatch_run(args),
        "docs" => dispatch_docs(args),
        "docs_build" => dispatch_docs_build(),
        "scene_inspect" => dispatch_scene_inspect(args),
        "sub_resource_add" => dispatch_sub_resource_add(args),
        "sub_resource_edit" => dispatch_sub_resource_edit(args),
        "connection_add" => dispatch_connection_add(args),
        "connection_remove" => dispatch_connection_remove(args),
        _ => Err(anyhow::anyhow!("Unknown tool: {}", name)),
    };

    let captured = output::end_capture();

    match result {
        Ok(ok) => {
            if captured.is_empty() {
                // Command didn't emit JSON — produce a fallback
                ToolResult {
                    is_error: !ok,
                    text: if ok {
                        r#"{"ok":true}"#.to_string()
                    } else {
                        r#"{"ok":false}"#.to_string()
                    },
                }
            } else {
                ToolResult {
                    is_error: !ok,
                    text: captured,
                }
            }
        }
        Err(e) => {
            if !captured.is_empty() {
                // Error happened after some output — return what was captured
                ToolResult {
                    is_error: true,
                    text: captured,
                }
            } else {
                ToolResult {
                    is_error: true,
                    text: format!(r#"{{"ok":false,"error":"{}"}}"#, escape_json(&format!("{e:#}"))),
                }
            }
        }
    }
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn str_arg(args: &Value, key: &str) -> Option<String> {
    args.get(key).and_then(|v| v.as_str()).map(String::from)
}

fn bool_arg(args: &Value, key: &str) -> bool {
    args.get(key).and_then(|v| v.as_bool()).unwrap_or(false)
}

fn str_array_arg(args: &Value, key: &str) -> Vec<String> {
    args.get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

// --- Dispatchers for commands that don't need Godot ---

fn dispatch_project_info() -> anyhow::Result<bool> {
    commands::project::run_info(true)
}

fn dispatch_scene_list() -> anyhow::Result<bool> {
    commands::scene::run_list(true)
}

fn dispatch_scene_validate(args: &Value) -> anyhow::Result<bool> {
    let path = str_arg(args, "path").ok_or_else(|| anyhow::anyhow!("missing required arg: path"))?;
    commands::scene::run_validate(&path, true)
}

fn dispatch_scene_create(args: &Value) -> anyhow::Result<bool> {
    let path =
        str_arg(args, "path").ok_or_else(|| anyhow::anyhow!("missing required arg: path"))?;
    let root_type = str_arg(args, "root_type")
        .ok_or_else(|| anyhow::anyhow!("missing required arg: root_type"))?;
    let root_name = str_arg(args, "root_name");
    let script = str_arg(args, "script");
    let force = bool_arg(args, "force");
    commands::scene::run_create(&path, &root_type, root_name.as_deref(), script.as_deref(), force, true)
}

fn dispatch_scene_edit(args: &Value) -> anyhow::Result<bool> {
    let path =
        str_arg(args, "path").ok_or_else(|| anyhow::anyhow!("missing required arg: path"))?;
    let set = str_array_arg(args, "set");
    if set.is_empty() {
        anyhow::bail!("missing required arg: set");
    }
    commands::scene::run_edit(&path, &set, true)
}

fn dispatch_scene_inspect(args: &Value) -> anyhow::Result<bool> {
    let path =
        str_arg(args, "path").ok_or_else(|| anyhow::anyhow!("missing required arg: path"))?;
    let node = str_arg(args, "node");
    commands::scene::run_inspect(&path, node.as_deref(), true)
}

fn dispatch_sub_resource_add(args: &Value) -> anyhow::Result<bool> {
    let scene =
        str_arg(args, "scene").ok_or_else(|| anyhow::anyhow!("missing required arg: scene"))?;
    let resource_type = str_arg(args, "resource_type")
        .ok_or_else(|| anyhow::anyhow!("missing required arg: resource_type"))?;
    let wire_node = str_arg(args, "wire_node");
    let wire_property = str_arg(args, "wire_property");

    let props_raw = str_array_arg(args, "props");
    let parsed_props: Vec<(String, String)> = props_raw
        .iter()
        .filter_map(|p| {
            let parts: Vec<&str> = p.splitn(2, '=').collect();
            if parts.len() == 2 {
                Some((
                    parts[0].to_string(),
                    scene_parser::format_prop_value(parts[1]),
                ))
            } else {
                None
            }
        })
        .collect();

    commands::sub_resource::run_add(
        &scene,
        &resource_type,
        &parsed_props,
        wire_node.as_deref(),
        wire_property.as_deref(),
        true,
    )
}

fn dispatch_sub_resource_edit(args: &Value) -> anyhow::Result<bool> {
    let scene =
        str_arg(args, "scene").ok_or_else(|| anyhow::anyhow!("missing required arg: scene"))?;
    let id = str_arg(args, "id").ok_or_else(|| anyhow::anyhow!("missing required arg: id"))?;
    let set = str_array_arg(args, "set");
    if set.is_empty() {
        anyhow::bail!("missing required arg: set");
    }
    commands::sub_resource::run_edit(&scene, &id, &set, true)
}

fn dispatch_node_add(args: &Value) -> anyhow::Result<bool> {
    let scene =
        str_arg(args, "scene").ok_or_else(|| anyhow::anyhow!("missing required arg: scene"))?;
    let node_type = str_arg(args, "node_type");
    let name =
        str_arg(args, "name").ok_or_else(|| anyhow::anyhow!("missing required arg: name"))?;
    let parent = str_arg(args, "parent");
    let script = str_arg(args, "script");
    let instance = str_arg(args, "instance");

    if node_type.is_none() && instance.is_none() {
        anyhow::bail!("Either node_type or instance must be provided");
    }

    if node_type.as_deref() == Some("PackedScene") {
        anyhow::bail!(
            "PackedScene is not a node type — it's a resource.\n\
             To instance a scene, use the 'instance' parameter instead:\n  \
             {{\"instance\": \"res://path/to/scene.tscn\"}}"
        );
    }

    let props_raw = str_array_arg(args, "props");
    let parsed_props: Vec<(String, String)> = props_raw
        .iter()
        .filter_map(|p| {
            let parts: Vec<&str> = p.splitn(2, '=').collect();
            if parts.len() == 2 {
                Some((
                    parts[0].to_string(),
                    scene_parser::format_prop_value(parts[1]),
                ))
            } else {
                None
            }
        })
        .collect();

    let sub_resource_type = str_arg(args, "sub_resource_type");
    let sub_resource_property = str_arg(args, "sub_resource_property");
    let sub_props_raw = str_array_arg(args, "sub_resource_props");
    let parsed_sub_props: Vec<(String, String)> = sub_props_raw
        .iter()
        .filter_map(|p| {
            let parts: Vec<&str> = p.splitn(2, '=').collect();
            if parts.len() == 2 {
                Some((
                    parts[0].to_string(),
                    scene_parser::format_prop_value(parts[1]),
                ))
            } else {
                None
            }
        })
        .collect();

    commands::node::run_add(
        &scene,
        node_type.as_deref(),
        &name,
        parent.as_deref(),
        script.as_deref(),
        &parsed_props,
        instance.as_deref(),
        sub_resource_type.as_deref(),
        &parsed_sub_props,
        sub_resource_property.as_deref(),
        true,
    )
}

fn dispatch_node_remove(args: &Value) -> anyhow::Result<bool> {
    let scene =
        str_arg(args, "scene").ok_or_else(|| anyhow::anyhow!("missing required arg: scene"))?;
    let name =
        str_arg(args, "name").ok_or_else(|| anyhow::anyhow!("missing required arg: name"))?;
    commands::node::run_remove(&scene, &name, true)
}

fn dispatch_uid_fix(args: &Value) -> anyhow::Result<bool> {
    let dry_run = bool_arg(args, "dry_run");
    commands::uid::run_fix(dry_run, true)
}

fn dispatch_script_create(args: &Value) -> anyhow::Result<bool> {
    let path =
        str_arg(args, "path").ok_or_else(|| anyhow::anyhow!("missing required arg: path"))?;
    let extends = str_arg(args, "extends").unwrap_or_else(|| "Node".to_string());
    let methods = str_array_arg(args, "methods");
    let force = bool_arg(args, "force");
    commands::script::run_create(&path, &extends, &methods, force, true)
}

fn dispatch_docs(args: &Value) -> anyhow::Result<bool> {
    let class =
        str_arg(args, "class").ok_or_else(|| anyhow::anyhow!("missing required arg: class"))?;
    let member = str_arg(args, "member");
    let members = bool_arg(args, "members");
    commands::docs::run_docs(&class, member.as_deref(), members, true)
}

fn dispatch_connection_add(args: &Value) -> anyhow::Result<bool> {
    let scene =
        str_arg(args, "scene").ok_or_else(|| anyhow::anyhow!("missing required arg: scene"))?;
    let signal =
        str_arg(args, "signal").ok_or_else(|| anyhow::anyhow!("missing required arg: signal"))?;
    let from =
        str_arg(args, "from").ok_or_else(|| anyhow::anyhow!("missing required arg: from"))?;
    let to = str_arg(args, "to").ok_or_else(|| anyhow::anyhow!("missing required arg: to"))?;
    let method =
        str_arg(args, "method").ok_or_else(|| anyhow::anyhow!("missing required arg: method"))?;
    commands::connection::run_add(&scene, &signal, &from, &to, &method, true)
}

fn dispatch_connection_remove(args: &Value) -> anyhow::Result<bool> {
    let scene =
        str_arg(args, "scene").ok_or_else(|| anyhow::anyhow!("missing required arg: scene"))?;
    let signal =
        str_arg(args, "signal").ok_or_else(|| anyhow::anyhow!("missing required arg: signal"))?;
    let from =
        str_arg(args, "from").ok_or_else(|| anyhow::anyhow!("missing required arg: from"))?;
    let to = str_arg(args, "to").ok_or_else(|| anyhow::anyhow!("missing required arg: to"))?;
    let method =
        str_arg(args, "method").ok_or_else(|| anyhow::anyhow!("missing required arg: method"))?;
    commands::connection::run_remove(&scene, &signal, &from, &to, &method, true)
}

// --- Dispatchers for commands that need Godot ---

fn dispatch_doctor() -> anyhow::Result<bool> {
    let godot_info = godot_finder::find_and_probe()?;
    commands::doctor::run(&godot_info, true)
}

fn dispatch_script_lint(args: &Value) -> anyhow::Result<bool> {
    let godot_info = godot_finder::find_and_probe()?;
    let file = str_arg(args, "file");
    commands::script::run_lint(&godot_info, file.as_deref(), true)
}

fn dispatch_run(args: &Value) -> anyhow::Result<bool> {
    let godot_info = godot_finder::find_and_probe()?;
    let timeout = args
        .get("timeout")
        .and_then(|v| v.as_u64())
        .unwrap_or(30);
    let scene = str_arg(args, "scene");
    commands::run::run_project(&godot_info, timeout, scene.as_deref(), true)
}

fn dispatch_docs_build() -> anyhow::Result<bool> {
    commands::docs::run_build(true)
}
