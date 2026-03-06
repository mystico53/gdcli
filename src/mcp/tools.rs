use serde_json::{json, Value};

pub struct ToolDef {
    pub name: &'static str,
    pub description: &'static str,
    pub schema: Value,
}

pub fn all_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "doctor",
            description: "Check Godot installation and project health",
            schema: json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        },
        ToolDef {
            name: "project_info",
            description: "Show project metadata (name, main scene, autoloads, file counts)",
            schema: json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        },
        ToolDef {
            name: "scene_list",
            description: "List all .tscn scenes with node and resource counts",
            schema: json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        },
        ToolDef {
            name: "scene_validate",
            description: "Validate a scene file for broken resource references",
            schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the .tscn file"
                    }
                },
                "required": ["path"],
                "additionalProperties": false
            }),
        },
        ToolDef {
            name: "scene_create",
            description: "Create a new .tscn scene file with a root node",
            schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path for the new scene file"
                    },
                    "root_type": {
                        "type": "string",
                        "description": "Type of the root node (e.g. Node2D, Control)"
                    },
                    "force": {
                        "type": "boolean",
                        "description": "Overwrite if file already exists",
                        "default": false
                    }
                },
                "required": ["path", "root_type"],
                "additionalProperties": false
            }),
        },
        ToolDef {
            name: "scene_edit",
            description: "Edit node properties in a scene file",
            schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the .tscn file"
                    },
                    "set": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Property edits in NodeName::property=value format"
                    }
                },
                "required": ["path", "set"],
                "additionalProperties": false
            }),
        },
        ToolDef {
            name: "node_add",
            description: "Add a node to a scene file",
            schema: json!({
                "type": "object",
                "properties": {
                    "scene": {
                        "type": "string",
                        "description": "Path to the .tscn file"
                    },
                    "node_type": {
                        "type": "string",
                        "description": "Node type (e.g. Sprite2D, Timer)"
                    },
                    "name": {
                        "type": "string",
                        "description": "Name for the new node"
                    },
                    "parent": {
                        "type": "string",
                        "description": "Parent node name (default: root node)"
                    },
                    "script": {
                        "type": "string",
                        "description": "Attach a script (res:// path)"
                    },
                    "props": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Properties as key=val strings"
                    }
                },
                "required": ["scene", "node_type", "name"],
                "additionalProperties": false
            }),
        },
        ToolDef {
            name: "node_remove",
            description: "Remove a node (and its children) from a scene file",
            schema: json!({
                "type": "object",
                "properties": {
                    "scene": {
                        "type": "string",
                        "description": "Path to the .tscn file"
                    },
                    "name": {
                        "type": "string",
                        "description": "Name of the node to remove"
                    }
                },
                "required": ["scene", "name"],
                "additionalProperties": false
            }),
        },
        ToolDef {
            name: "uid_fix",
            description: "Fix stale UID references in scene/resource files",
            schema: json!({
                "type": "object",
                "properties": {
                    "dry_run": {
                        "type": "boolean",
                        "description": "Show what would change without applying",
                        "default": false
                    }
                },
                "additionalProperties": false
            }),
        },
        ToolDef {
            name: "script_create",
            description: "Create a new GDScript file with boilerplate",
            schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path for the new script file"
                    },
                    "extends": {
                        "type": "string",
                        "description": "Base class to extend (default: Node)",
                        "default": "Node"
                    },
                    "methods": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Lifecycle methods to include (e.g. _ready, _process)"
                    },
                    "force": {
                        "type": "boolean",
                        "description": "Overwrite if file already exists",
                        "default": false
                    }
                },
                "required": ["path"],
                "additionalProperties": false
            }),
        },
        ToolDef {
            name: "script_lint",
            description: "Check GDScript files for parse/compile errors",
            schema: json!({
                "type": "object",
                "properties": {
                    "file": {
                        "type": "string",
                        "description": "Check a single file instead of the whole project"
                    }
                },
                "additionalProperties": false
            }),
        },
        ToolDef {
            name: "run",
            // NOTE: This blocks the single-threaded event loop for up to `timeout` seconds.
            // No other MCP requests will be processed while a run is in progress.
            description: "Run the Godot project headlessly (blocks server until complete)",
            schema: json!({
                "type": "object",
                "properties": {
                    "timeout": {
                        "type": "integer",
                        "description": "Timeout in seconds (default: 30)",
                        "default": 30
                    },
                    "scene": {
                        "type": "string",
                        "description": "Scene path to run (default: main scene)"
                    }
                },
                "additionalProperties": false
            }),
        },
        ToolDef {
            name: "docs",
            description: "Look up Godot API documentation for a class or member",
            schema: json!({
                "type": "object",
                "properties": {
                    "class": {
                        "type": "string",
                        "description": "Class name to look up"
                    },
                    "member": {
                        "type": "string",
                        "description": "Specific member (method/property/signal) to look up"
                    },
                    "members": {
                        "type": "boolean",
                        "description": "List all methods, properties, and signals",
                        "default": false
                    }
                },
                "required": ["class"],
                "additionalProperties": false
            }),
        },
        ToolDef {
            name: "docs_build",
            description: "Build/rebuild the Godot API docs cache (runs godot --doctool)",
            schema: json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
        },
    ]
}

pub fn tools_list_json() -> Value {
    let tools: Vec<Value> = all_tools()
        .into_iter()
        .map(|t| {
            json!({
                "name": t.name,
                "description": t.description,
                "inputSchema": t.schema
            })
        })
        .collect();
    json!({ "tools": tools })
}
