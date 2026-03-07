mod commands;
mod docs_parser;
mod errors;
mod godot_finder;
mod mcp;
mod output;
mod project_util;
mod runner;
mod scene_parser;
mod session;

use clap::{Parser, Subcommand};
use std::io::IsTerminal;

#[derive(Parser)]
#[command(name = "gdcli", version, about = "Agent-friendly CLI for Godot 4")]
struct Cli {
    /// Output JSON instead of human-readable text
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check Godot installation and project health
    Doctor,

    /// GDScript operations (lint, create)
    Script {
        #[command(subcommand)]
        action: ScriptAction,
    },

    /// Run the project headlessly
    Run {
        /// Timeout in seconds (default: 30)
        #[arg(long, default_value = "30")]
        timeout: u64,

        /// Scene path to run (default: main scene)
        #[arg(long)]
        scene: Option<String>,
    },

    /// Show project information
    Project {
        #[command(subcommand)]
        action: ProjectAction,
    },

    /// Scene operations (list, validate, create, edit, inspect)
    Scene {
        #[command(subcommand)]
        action: SceneAction,
    },

    /// Sub-resource operations (add, edit)
    SubResource {
        #[command(subcommand)]
        action: SubResourceAction,
    },

    /// Node operations (add, remove)
    Node {
        #[command(subcommand)]
        action: NodeAction,
    },

    /// Signal connection operations (add, remove)
    Connection {
        #[command(subcommand)]
        action: ConnectionAction,
    },

    /// Add a Sprite2D/Sprite3D node with a texture in one call
    LoadSprite {
        /// Path to the .tscn file
        scene: String,

        /// Name for the new sprite node
        name: String,

        /// Texture resource path (e.g. res://icon.svg)
        texture: String,

        /// Sprite type: Sprite2D (default) or Sprite3D
        #[arg(long, default_value = "Sprite2D")]
        sprite_type: String,

        /// Parent node name (default: root node)
        #[arg(long)]
        parent: Option<String>,

        /// Properties as key=val pairs (semicolon-separated)
        #[arg(long, value_delimiter = ';')]
        props: Vec<String>,
    },

    /// Fix stale UID references
    Uid {
        #[command(subcommand)]
        action: UidAction,
    },

    /// Start MCP server (JSON-RPC over stdio)
    Mcp {
        /// Set the working directory before starting the server
        #[arg(long)]
        project_dir: Option<String>,
    },

    /// Godot API documentation lookup
    Docs {
        /// Class name to look up
        #[arg(required_unless_present = "build")]
        class: Option<String>,

        /// Specific member (method/property/signal) to look up
        member: Option<String>,

        /// List all methods, properties, and signals
        #[arg(long)]
        members: bool,

        /// Build/rebuild docs cache by running `godot --doctool`
        #[arg(long)]
        build: bool,
    },
}

#[derive(Subcommand)]
enum ScriptAction {
    /// Check scripts for parse errors
    Lint {
        /// Check a single file instead of the whole project
        #[arg(long)]
        file: Option<String>,
    },

    /// Create a new GDScript file with boilerplate
    Create {
        /// Path for the new script file
        path: String,

        /// Base class to extend (default: Node)
        #[arg(long, default_value = "Node")]
        extends: String,

        /// Comma-separated lifecycle methods to include
        #[arg(long, value_delimiter = ',')]
        methods: Vec<String>,

        /// Overwrite if file already exists
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand)]
enum ProjectAction {
    /// Display project metadata
    Info,

    /// Initialize a new Godot project (creates project.godot)
    Init {
        /// Directory to create the project in (default: current directory)
        #[arg(long)]
        path: Option<String>,

        /// Project name (default: directory name)
        #[arg(long)]
        name: Option<String>,

        /// Godot version (e.g. "4.6") — auto-detected if Godot is installed
        #[arg(long)]
        godot_version: Option<String>,

        /// Renderer: forward_plus (default), mobile, or gl_compatibility
        #[arg(long)]
        renderer: Option<String>,

        /// Overwrite existing project.godot
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand)]
enum SceneAction {
    /// List all scenes with node counts
    List,

    /// Validate a scene for broken references
    Validate {
        /// Path to the .tscn file
        path: String,
    },

    /// Create a new .tscn scene file
    Create {
        /// Path for the new scene file
        path: String,

        /// Type of the root node
        #[arg(long)]
        root_type: String,

        /// Name for the root node (default: derived from filename)
        #[arg(long)]
        root_name: Option<String>,

        /// Attach a script to the root node (res:// path)
        #[arg(long)]
        script: Option<String>,

        /// Overwrite if file already exists
        #[arg(long)]
        force: bool,
    },

    /// Edit node properties in a scene file
    Edit {
        /// Path to the .tscn file
        path: String,

        /// Property edits in NodeName::property=value format
        #[arg(long = "set", required = true)]
        set: Vec<String>,
    },

    /// Inspect a scene file (show all nodes, resources, connections)
    Inspect {
        /// Path to the .tscn file
        path: String,

        /// Filter to a single node (includes only its referenced sub_resources and ext_resources)
        #[arg(long)]
        node: Option<String>,
    },
}

#[derive(Subcommand)]
enum SubResourceAction {
    /// Add a sub_resource to a scene file
    Add {
        /// Path to the .tscn file
        scene: String,

        /// Resource type (e.g. RectangleShape2D, CircleShape2D, BoxMesh)
        resource_type: String,

        /// Properties as key=val pairs (semicolon-separated)
        #[arg(long, value_delimiter = ';')]
        props: Vec<String>,

        /// Wire to this node by setting its property to SubResource("id")
        #[arg(long)]
        wire_node: Option<String>,

        /// Property on the wire_node to set (required if wire_node is set)
        #[arg(long)]
        wire_property: Option<String>,
    },

    /// Edit properties on an existing sub_resource
    Edit {
        /// Path to the .tscn file
        scene: String,

        /// Sub-resource ID to edit
        id: String,

        /// Property edits as key=value pairs
        #[arg(long = "set", required = true)]
        set: Vec<String>,
    },
}

#[derive(Subcommand)]
enum NodeAction {
    /// Add a node to a scene file
    Add {
        /// Path to the .tscn file
        scene: String,

        /// Node type (e.g. Sprite2D, Timer, Node2D) — required unless --instance is used
        #[arg(long = "node-type")]
        node_type: Option<String>,

        /// Name for the new node
        #[arg(long)]
        name: String,

        /// Parent node name (default: root node)
        #[arg(long)]
        parent: Option<String>,

        /// Attach a script (res:// path)
        #[arg(long)]
        script: Option<String>,

        /// Instance a scene instead of creating a typed node (res:// path)
        #[arg(long)]
        instance: Option<String>,

        /// Properties as key=val pairs (semicolon-separated)
        #[arg(long, value_delimiter = ';')]
        props: Vec<String>,

        /// Create an inline sub_resource of this type and wire it to the node
        #[arg(long = "sub-resource")]
        sub_resource: Option<String>,

        /// Properties for the sub_resource as key=val pairs (semicolon-separated)
        #[arg(long = "sub-resource-props", value_delimiter = ';')]
        sub_resource_props: Vec<String>,

        /// Property on the node to wire the sub_resource to (inferred from node type if not set)
        #[arg(long = "sub-resource-property")]
        sub_resource_property: Option<String>,
    },

    /// Remove a node (and its children) from a scene file
    Remove {
        /// Path to the .tscn file
        scene: String,

        /// Name of the node to remove
        name: String,
    },

    /// Reorder a node within a scene file (controls draw/process order)
    Reorder {
        /// Path to the .tscn file
        scene: String,

        /// Name of the node to move
        name: String,

        /// 0-based position among siblings
        #[arg(long)]
        position: Option<String>,

        /// Move before this node
        #[arg(long)]
        before: Option<String>,

        /// Move after this node
        #[arg(long)]
        after: Option<String>,
    },
}

#[derive(Subcommand)]
enum ConnectionAction {
    /// Add a signal connection between nodes
    Add {
        /// Path to the .tscn file
        scene: String,

        /// Signal name (e.g. pressed, timeout, body_entered)
        signal: String,

        /// Source node name (emitter) — use "." for root
        from: String,

        /// Target node name (receiver) — use "." for root
        to: String,

        /// Method name on the target node
        method: String,
    },

    /// Remove a signal connection
    Remove {
        /// Path to the .tscn file
        scene: String,

        /// Signal name
        signal: String,

        /// Source node name — use "." for root
        from: String,

        /// Target node name — use "." for root
        to: String,

        /// Method name on the target node
        method: String,
    },
}

#[derive(Subcommand)]
enum UidAction {
    /// Fix stale UID references in scene/resource files
    Fix {
        /// Show what would change without applying
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() {
    // When double-clicked (no args, interactive terminal), show a friendly message
    if std::env::args().len() <= 1 && std::io::stdout().is_terminal() {
        println!("gdcli - CLI toolkit for Godot 4");
        println!();
        println!("This is a command-line tool. Open a terminal and run:");
        println!("  gdcli doctor        Check your setup");
        println!("  gdcli --help        See all commands");
        println!();
        println!("Press Enter to exit...");
        let _ = std::io::stdin().read_line(&mut String::new());
        return;
    }

    let cli = Cli::parse();
    let json_mode = output::use_json(cli.json);

    if let Err(err) = run(cli.command, json_mode) {
        if json_mode {
            let envelope: output::JsonEnvelope<()> = output::JsonEnvelope {
                ok: false,
                command: "unknown".into(),
                data: None,
                error: Some(format!("{err:#}")),
            };
            output::emit_json(&envelope);
        } else {
            output::print_error(&format!("{err:#}"));
        }
        std::process::exit(1);
    }
}

fn run(command: Commands, json_mode: bool) -> anyhow::Result<()> {
    // MCP server mode — takes over stdio, never returns normally
    if let Commands::Mcp { project_dir } = command {
        return mcp::run_mcp_server(project_dir.as_deref());
    }

    // Commands that don't need Godot (pure filesystem)
    match &command {
        Commands::Project {
            action: ProjectAction::Info,
        } => {
            let ok = commands::project::run_info(json_mode)?;
            if !ok {
                std::process::exit(1);
            }
            return Ok(());
        }
        Commands::Project {
            action:
                ProjectAction::Init {
                    path,
                    name,
                    godot_version,
                    renderer,
                    force,
                },
        } => {
            let ok = commands::project::run_init(
                path.as_deref(),
                name.as_deref(),
                godot_version.as_deref(),
                renderer.as_deref(),
                *force,
                json_mode,
            )?;
            if !ok {
                std::process::exit(1);
            }
            return Ok(());
        }
        Commands::Scene { action } => {
            let ok = match action {
                SceneAction::List => commands::scene::run_list(json_mode)?,
                SceneAction::Validate { path } => commands::scene::run_validate(path, json_mode)?,
                SceneAction::Create {
                    path,
                    root_type,
                    root_name,
                    script,
                    force,
                } => commands::scene::run_create(
                    path,
                    root_type,
                    root_name.as_deref(),
                    script.as_deref(),
                    *force,
                    json_mode,
                )?,
                SceneAction::Edit { path, set } => commands::scene::run_edit(path, set, json_mode)?,
                SceneAction::Inspect { path, node } => {
                    commands::scene::run_inspect(path, node.as_deref(), json_mode)?
                }
            };
            if !ok {
                std::process::exit(1);
            }
            return Ok(());
        }
        Commands::SubResource { action } => {
            let ok = match action {
                SubResourceAction::Add {
                    scene,
                    resource_type,
                    props,
                    wire_node,
                    wire_property,
                } => {
                    let parsed_props: Vec<(String, String)> = props
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
                        scene,
                        resource_type,
                        &parsed_props,
                        wire_node.as_deref(),
                        wire_property.as_deref(),
                        json_mode,
                    )?
                }
                SubResourceAction::Edit { scene, id, set } => {
                    commands::sub_resource::run_edit(scene, id, set, json_mode)?
                }
            };
            if !ok {
                std::process::exit(1);
            }
            return Ok(());
        }
        Commands::Node { action } => {
            let ok = match action {
                NodeAction::Add {
                    scene,
                    node_type,
                    name,
                    parent,
                    script,
                    instance,
                    props,
                    sub_resource,
                    sub_resource_props,
                    sub_resource_property,
                } => {
                    if node_type.is_none() && instance.is_none() {
                        anyhow::bail!("Either <NODE_TYPE> or --instance must be provided");
                    }
                    let parsed_props: Vec<(String, String)> = props
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
                    let parsed_sub_props: Vec<(String, String)> = sub_resource_props
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
                        scene,
                        node_type.as_deref(),
                        name,
                        parent.as_deref(),
                        script.as_deref(),
                        &parsed_props,
                        instance.as_deref(),
                        sub_resource.as_deref(),
                        &parsed_sub_props,
                        sub_resource_property.as_deref(),
                        json_mode,
                    )?
                }
                NodeAction::Remove { scene, name } => {
                    commands::node::run_remove(scene, name, json_mode)?
                }
                NodeAction::Reorder {
                    scene,
                    name,
                    position,
                    before,
                    after,
                } => commands::node::run_reorder(
                    scene,
                    name,
                    position.as_deref(),
                    before.as_deref(),
                    after.as_deref(),
                    json_mode,
                )?
            };
            if !ok {
                std::process::exit(1);
            }
            return Ok(());
        }
        Commands::Connection { action } => {
            let ok = match action {
                ConnectionAction::Add {
                    scene,
                    signal,
                    from,
                    to,
                    method,
                } => commands::connection::run_add(scene, signal, from, to, method, json_mode)?,
                ConnectionAction::Remove {
                    scene,
                    signal,
                    from,
                    to,
                    method,
                } => commands::connection::run_remove(scene, signal, from, to, method, json_mode)?,
            };
            if !ok {
                std::process::exit(1);
            }
            return Ok(());
        }
        Commands::LoadSprite {
            scene,
            name,
            texture,
            sprite_type,
            parent,
            props,
        } => {
            let parsed_props: Vec<(String, String)> = props
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
            let ok = commands::sprite::run_load_sprite(
                scene,
                name,
                texture,
                Some(sprite_type.as_str()),
                parent.as_deref(),
                &parsed_props,
                json_mode,
            )?;
            if !ok {
                std::process::exit(1);
            }
            return Ok(());
        }
        Commands::Uid {
            action: UidAction::Fix { dry_run },
        } => {
            let ok = commands::uid::run_fix(*dry_run, json_mode)?;
            if !ok {
                std::process::exit(1);
            }
            return Ok(());
        }
        Commands::Script {
            action:
                ScriptAction::Create {
                    path,
                    extends,
                    methods,
                    force,
                },
        } => {
            let ok = commands::script::run_create(path, extends, methods, *force, json_mode)?;
            if !ok {
                std::process::exit(1);
            }
            return Ok(());
        }
        Commands::Docs { build: true, .. } => {
            let ok = commands::docs::run_build(json_mode)?;
            if !ok {
                std::process::exit(1);
            }
            return Ok(());
        }
        Commands::Docs {
            class: Some(class),
            member,
            members,
            build: false,
        } => {
            let ok = commands::docs::run_docs(class, member.as_deref(), *members, json_mode)?;
            if !ok {
                std::process::exit(1);
            }
            return Ok(());
        }
        _ => {}
    }

    // Commands that need Godot
    let godot_info = match godot_finder::find_and_probe() {
        Ok(info) => info,
        Err(err) => {
            let cmd_name = match &command {
                Commands::Doctor => "doctor",
                Commands::Script { .. } => "script lint",
                Commands::Run { .. } => "run",
                _ => "unknown",
            };
            if json_mode {
                let envelope: output::JsonEnvelope<()> = output::JsonEnvelope {
                    ok: false,
                    command: cmd_name.into(),
                    data: None,
                    error: Some(format!("{err:#}")),
                };
                output::emit_json(&envelope);
                std::process::exit(1);
            }
            return Err(err);
        }
    };

    let ok = match command {
        Commands::Doctor => commands::doctor::run(&godot_info, json_mode)?,
        Commands::Script {
            action: ScriptAction::Lint { file },
        } => commands::script::run_lint(&godot_info, file.as_deref(), json_mode)?,
        Commands::Run { timeout, scene } => {
            commands::run::run_project(&godot_info, timeout, scene.as_deref(), json_mode)?
        }
        _ => unreachable!(),
    };

    if !ok {
        std::process::exit(1);
    }

    Ok(())
}
