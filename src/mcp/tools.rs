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
            name: "project_init",
            description: "Initialize a new Godot project (creates project.godot with config_version=5 and proper features). Auto-detects Godot version if installed.",
            schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Directory to create the project in (default: current directory)"
                    },
                    "name": {
                        "type": "string",
                        "description": "Project name (default: directory name)"
                    },
                    "godot_version": {
                        "type": "string",
                        "description": "Godot version e.g. \"4.6\" (auto-detected if Godot is installed)"
                    },
                    "renderer": {
                        "type": "string",
                        "description": "Renderer: forward_plus (default), mobile, or gl_compatibility"
                    },
                    "force": {
                        "type": "boolean",
                        "description": "Overwrite existing project.godot",
                        "default": false
                    }
                },
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
                        "description": "Path to the .tscn file (accepts res:// paths)"
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
                        "description": "Path for the new scene file (accepts res:// paths)"
                    },
                    "root_type": {
                        "type": "string",
                        "description": "Type of the root node (e.g. Node2D, Control)"
                    },
                    "root_name": {
                        "type": "string",
                        "description": "Name for the root node (default: derived from filename, e.g. enemy.tscn -> Enemy)"
                    },
                    "script": {
                        "type": "string",
                        "description": "Attach a script to the root node (res:// path)"
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
                        "description": "Path to the .tscn file (accepts res:// paths)"
                    },
                    "set": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Property edits in NodeName::property=value format. String values are auto-quoted. Use slash syntax for theme overrides: `Label::theme_override_font_sizes/font_size=28`"
                    }
                },
                "required": ["path", "set"],
                "additionalProperties": false
            }),
        },
        ToolDef {
            name: "node_add",
            description: "Add a typed node or instanced scene to a scene file. Provide either node_type or instance (not both). Use sub_resource_type to create an inline sub_resource and wire it to the node (e.g. CollisionShape2D + RectangleShape2D in one call).",
            schema: json!({
                "type": "object",
                "properties": {
                    "scene": {
                        "type": "string",
                        "description": "Path to the .tscn file (accepts res:// paths)"
                    },
                    "node_type": {
                        "type": "string",
                        "description": "Node type (e.g. Sprite2D, Timer) — required unless instance is provided"
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
                        "description": "Attach a script (res:// path) — only for typed nodes"
                    },
                    "instance": {
                        "type": "string",
                        "description": "Instance a scene (res:// path) instead of creating a typed node"
                    },
                    "props": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Properties as key=val strings (one per array element). String values are auto-quoted — do NOT wrap in quotes. Use slash syntax for theme overrides: `theme_override_font_sizes/font_size=28`"
                    },
                    "sub_resource_type": {
                        "type": "string",
                        "description": "Create an inline sub_resource of this type and wire it to the node (e.g. RectangleShape2D, CircleShape2D)"
                    },
                    "sub_resource_props": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Properties for the sub_resource as key=val strings (e.g. [\"size=Vector2(30,30)\"])"
                    },
                    "sub_resource_property": {
                        "type": "string",
                        "description": "Property on the node to wire the sub_resource to (auto-inferred for common types like CollisionShape2D→shape, MeshInstance3D→mesh)"
                    }
                },
                "required": ["scene", "name"],
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
                        "description": "Path to the .tscn file (accepts res:// paths)"
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
                        "description": "Path for the new script file (accepts res:// paths)"
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
                        "description": "Check a single file instead of the whole project (accepts res:// paths)"
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
                        "description": "Scene path to run (default: main scene, accepts res:// paths)"
                    }
                },
                "additionalProperties": false
            }),
        },
        ToolDef {
            name: "run_start",
            description: "Start a Godot project running headlessly in the background. Returns a session_id to poll with run_read or stop with run_stop. Does NOT block the server.",
            schema: json!({
                "type": "object",
                "properties": {
                    "timeout": {
                        "type": "integer",
                        "description": "Timeout in seconds (default: 30). Process is killed if it exceeds this.",
                        "default": 30
                    },
                    "scene": {
                        "type": "string",
                        "description": "Scene path to run (default: main scene, accepts res:// paths)"
                    }
                },
                "additionalProperties": false
            }),
        },
        ToolDef {
            name: "run_read",
            description: "Poll a running session for new stdout/stderr output since last read. Returns session status (running, exited, timed_out, killed) and incremental output.",
            schema: json!({
                "type": "object",
                "properties": {
                    "session_id": {
                        "type": "string",
                        "description": "Session ID returned by run_start"
                    }
                },
                "required": ["session_id"],
                "additionalProperties": false
            }),
        },
        ToolDef {
            name: "run_stop",
            description: "Stop a running session (kills process if still running) and return all accumulated output. The session is removed after this call.",
            schema: json!({
                "type": "object",
                "properties": {
                    "session_id": {
                        "type": "string",
                        "description": "Session ID returned by run_start"
                    }
                },
                "required": ["session_id"],
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
        ToolDef {
            name: "scene_inspect",
            description: "Inspect a scene file — returns nodes, ext_resources, sub_resources (with properties), and connections. Use --node to filter to a single node and its referenced resources.",
            schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the .tscn file (accepts res:// paths)"
                    },
                    "node": {
                        "type": "string",
                        "description": "Filter to a single node by name — only returns that node plus its referenced sub_resources and ext_resources"
                    }
                },
                "required": ["path"],
                "additionalProperties": false
            }),
        },
        ToolDef {
            name: "sub_resource_add",
            description: "Add a sub_resource (e.g. shape, material) to a scene file, optionally wiring it to a node property",
            schema: json!({
                "type": "object",
                "properties": {
                    "scene": {
                        "type": "string",
                        "description": "Path to the .tscn file (accepts res:// paths)"
                    },
                    "resource_type": {
                        "type": "string",
                        "description": "Resource type (e.g. RectangleShape2D, CircleShape2D, BoxMesh, StandardMaterial3D)"
                    },
                    "props": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Properties as key=val strings (e.g. [\"size=Vector2(30,30)\"]). String values are auto-quoted."
                    },
                    "wire_node": {
                        "type": "string",
                        "description": "Node name to wire this sub_resource to (sets node property to SubResource(\"id\"))"
                    },
                    "wire_property": {
                        "type": "string",
                        "description": "Property on wire_node to set (required if wire_node is provided, e.g. \"shape\", \"mesh\", \"material\")"
                    }
                },
                "required": ["scene", "resource_type"],
                "additionalProperties": false
            }),
        },
        ToolDef {
            name: "sub_resource_edit",
            description: "Edit properties on an existing sub_resource in a scene file",
            schema: json!({
                "type": "object",
                "properties": {
                    "scene": {
                        "type": "string",
                        "description": "Path to the .tscn file (accepts res:// paths)"
                    },
                    "id": {
                        "type": "string",
                        "description": "Sub-resource ID to edit (e.g. \"RectangleShape2D_abc5x\")"
                    },
                    "set": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Property edits as key=value strings (e.g. [\"size=Vector2(50,50)\"]). String values are auto-quoted."
                    }
                },
                "required": ["scene", "id", "set"],
                "additionalProperties": false
            }),
        },
        ToolDef {
            name: "load_sprite",
            description: "Add a Sprite2D/Sprite3D node with a texture wired up in a single call. Validates the texture file exists on disk.",
            schema: json!({
                "type": "object",
                "properties": {
                    "scene": {
                        "type": "string",
                        "description": "Path to the .tscn file (accepts res:// paths)"
                    },
                    "name": {
                        "type": "string",
                        "description": "Name for the new sprite node"
                    },
                    "texture": {
                        "type": "string",
                        "description": "Texture resource path (e.g. res://icon.svg)"
                    },
                    "sprite_type": {
                        "type": "string",
                        "description": "Sprite2D (default) or Sprite3D",
                        "default": "Sprite2D"
                    },
                    "parent": {
                        "type": "string",
                        "description": "Parent node name (default: root node)"
                    },
                    "props": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Additional properties as key=val strings (e.g. [\"position=Vector2(100,200)\"]). String values are auto-quoted."
                    }
                },
                "required": ["scene", "name", "texture"],
                "additionalProperties": false
            }),
        },
        ToolDef {
            name: "connection_add",
            description: "Add a signal connection between nodes in a scene file",
            schema: json!({
                "type": "object",
                "properties": {
                    "scene": {
                        "type": "string",
                        "description": "Path to the .tscn file (accepts res:// paths)"
                    },
                    "signal": {
                        "type": "string",
                        "description": "Signal name (e.g. pressed, timeout, body_entered)"
                    },
                    "from": {
                        "type": "string",
                        "description": "Source node name (emitter) — use \".\" for root"
                    },
                    "to": {
                        "type": "string",
                        "description": "Target node name (receiver) — use \".\" for root"
                    },
                    "method": {
                        "type": "string",
                        "description": "Method name on the target node (e.g. _on_button_pressed)"
                    }
                },
                "required": ["scene", "signal", "from", "to", "method"],
                "additionalProperties": false
            }),
        },
        ToolDef {
            name: "connection_remove",
            description: "Remove a signal connection from a scene file",
            schema: json!({
                "type": "object",
                "properties": {
                    "scene": {
                        "type": "string",
                        "description": "Path to the .tscn file (accepts res:// paths)"
                    },
                    "signal": {
                        "type": "string",
                        "description": "Signal name"
                    },
                    "from": {
                        "type": "string",
                        "description": "Source node name — use \".\" for root"
                    },
                    "to": {
                        "type": "string",
                        "description": "Target node name — use \".\" for root"
                    },
                    "method": {
                        "type": "string",
                        "description": "Method name on the target node"
                    }
                },
                "required": ["scene", "signal", "from", "to", "method"],
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
