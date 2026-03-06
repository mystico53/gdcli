mod commands;
mod docs_parser;
mod errors;
mod godot_finder;
mod mcp;
mod output;
mod runner;
mod scene_parser;

use clap::{Parser, Subcommand};

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

    /// Scene operations (list, validate, create, edit)
    Scene {
        #[command(subcommand)]
        action: SceneAction,
    },

    /// Node operations (add, remove)
    Node {
        #[command(subcommand)]
        action: NodeAction,
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
}

#[derive(Subcommand)]
enum NodeAction {
    /// Add a node to a scene file
    Add {
        /// Path to the .tscn file
        scene: String,

        /// Node type (e.g. Sprite2D, Timer, Node2D)
        node_type: String,

        /// Name for the new node
        name: String,

        /// Parent node name (default: root node)
        #[arg(long)]
        parent: Option<String>,

        /// Attach a script (res:// path)
        #[arg(long)]
        script: Option<String>,

        /// Properties as key=val pairs
        #[arg(long, value_delimiter = ',')]
        props: Vec<String>,
    },

    /// Remove a node (and its children) from a scene file
    Remove {
        /// Path to the .tscn file
        scene: String,

        /// Name of the node to remove
        name: String,
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
        Commands::Scene { action } => {
            let ok = match action {
                SceneAction::List => commands::scene::run_list(json_mode)?,
                SceneAction::Validate { path } => commands::scene::run_validate(path, json_mode)?,
                SceneAction::Create {
                    path,
                    root_type,
                    force,
                } => commands::scene::run_create(path, root_type, *force, json_mode)?,
                SceneAction::Edit { path, set } => commands::scene::run_edit(path, set, json_mode)?,
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
                    commands::node::run_add(
                        scene,
                        node_type,
                        name,
                        parent.as_deref(),
                        script.as_deref(),
                        &parsed_props,
                        json_mode,
                    )?
                }
                NodeAction::Remove { scene, name } => {
                    commands::node::run_remove(scene, name, json_mode)?
                }
            };
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
