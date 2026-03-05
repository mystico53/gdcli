mod commands;
mod errors;
mod godot_finder;
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

    /// Lint GDScript files for parse errors
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

    /// List and validate scene files
    Scene {
        #[command(subcommand)]
        action: SceneAction,
    },

    /// Fix stale UID references
    Uid {
        #[command(subcommand)]
        action: UidAction,
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
