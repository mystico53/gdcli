# gdcli — Agent Developer Guide

Agent-friendly CLI for Godot 4. Provides structured, machine-readable output for linting GDScript, running projects headlessly, creating/editing scenes and scripts, looking up Godot API docs, managing UID references, and serving all commands over MCP.

## Project Overview

gdcli wraps a **stock Godot 4 binary** — no patched build needed. The CLI discovers the Godot binary dynamically (env var → PATH → common install paths) and probes its version at runtime. Error parsing uses Godot's standard `SCRIPT ERROR:` output format.

All commands emit a JSON envelope `{ ok, command, data, error }` when stdout is not a TTY or when `--json` is passed. This makes every command directly consumable by AI agents without extra parsing.

## Build & Test

```bash
# Build
cargo build

# Run tests
cargo test

# Lint with clippy
cargo clippy -- -D warnings

# Run a specific command
cargo run -- doctor
cargo run -- script lint --file path/to/script.gd
cargo run -- run --timeout 15
cargo run -- project info
cargo run -- scene list
cargo run -- scene validate path/to/scene.tscn
cargo run -- scene create path/to/new.tscn --root-type Node2D
cargo run -- scene edit path/to/scene.tscn --set Player::speed=200
cargo run -- node add scene.tscn Sprite2D MySprite
cargo run -- node remove scene.tscn MySprite
cargo run -- script create path/to/script.gd --extends Node --methods _ready
cargo run -- uid fix --dry-run
cargo run -- docs Node2D
cargo run -- docs --build
```

## Architecture

gdcli has a two-layer design:

| Layer | Commands | How it works |
|---|---|---|
| **Filesystem** | `project info`, `scene list/validate/create/edit`, `node add/remove`, `uid fix`, `script create`, `docs` | Pure file I/O — parses `.godot`, `.tscn`, `.tres`, `.uid`, and XML files directly. Instant, no Godot needed. |
| **Godot subprocess** | `doctor`, `script lint`, `run`, `docs --build` | Spawns the Godot binary with `--headless` and captures stdout/stderr. Works with stock Godot 4. |
| **MCP server** | `mcp` | JSON-RPC 2.0 over stdio — exposes all commands as MCP tools for AI clients. |

### JSON Envelope

Every command outputs the same envelope shape:

```json
{
  "ok": true,
  "command": "script lint",
  "data": { ... },
  "error": null
}
```

- `ok`: `true` if no errors/issues found, `false` otherwise
- `command`: the subcommand name
- `data`: command-specific payload (see each command module's report struct)
- `error`: human-readable error string, or `null`

JSON mode activates automatically when stdout is not a TTY (pipe, redirect, agent invocation). Force it with `--json`.

## Source Layout

| File | Purpose |
|---|---|
| `src/main.rs` | CLI entry point. Defines clap commands/subcommands, routes to handlers. Splits filesystem commands (no Godot needed) from subprocess commands. |
| `src/runner.rs` | Spawns Godot as a subprocess. `run()` adds `--headless`; `run_raw()` runs with exact args. Drains stdout/stderr in background threads to prevent pipe deadlocks. Handles timeouts via `wait-timeout`. |
| `src/godot_finder.rs` | Discovers Godot binary (`GODOT_PATH` → `which` → common paths). Probes `--version`. Prefers `.console.exe` on Windows. |
| `src/output.rs` | `JsonEnvelope<T>` struct, `use_json()` TTY detection, `emit_json()`, colored TTY helpers (`print_check`, `print_header`, `print_error`). |
| `src/errors.rs` | Parses Godot error output. `parse_script_errors()` handles `SCRIPT ERROR:` blocks from Godot's standard output. Filters noise errors (`.godot/` internal, global script cache). |
| `src/scene_parser.rs` | Parses `.tscn` files into `ParsedScene` (header, ext_resources, sub_resources, nodes, connections). Also provides `find_scene_files()` for recursive `.tscn` discovery. |
| `src/commands/mod.rs` | Module declarations for all command handlers. |
| `src/commands/doctor.rs` | `gdcli doctor` — checks Godot binary, `project.godot` presence, `.gd` file count. |
| `src/commands/script.rs` | `gdcli script lint` — single-file lint via `--check-only`, project-wide lint via `--headless --quit`. Both parse `SCRIPT ERROR:` blocks. |
| `src/commands/run.rs` | `gdcli run` — runs project headlessly with timeout. Reports exit code, stdout, stderr, parsed errors, timeout status. |
| `src/commands/project.rs` | `gdcli project info` — parses `project.godot` for name, main scene, autoloads, script/scene counts. |
| `src/commands/scene.rs` | `gdcli scene list` / `gdcli scene validate` — lists scenes with node counts, validates ext_resource paths exist on disk. |
| `src/commands/uid.rs` | `gdcli uid fix` — scans `.uid` files to build UID→path map, finds stale path refs in `.tscn`/`.tres`, fixes them. `--dry-run` supported. |
| `src/commands/docs.rs` | `gdcli docs` — looks up Godot API docs from cached XML class reference. `--build` runs `godot --doctool` to generate the cache. |
| `src/commands/node.rs` | `gdcli node add/remove` — adds or removes nodes in `.tscn` files. Handles ext_resource management, parent paths, script attachment, and property setting. |
| `src/docs_parser.rs` | Parses Godot XML class reference files (`doc/classes/*.xml`). Extracts methods, properties, signals, descriptions. |
| `src/mcp/mod.rs` | MCP server entry point. JSON-RPC 2.0 event loop over stdio — reads requests line-by-line, dispatches, writes responses. |
| `src/mcp/protocol.rs` | JSON-RPC 2.0 types: `JsonRpcRequest`, `JsonRpcResponse`, `JsonRpcError`. Handles serialization. |
| `src/mcp/tools.rs` | MCP tool definitions. 14 tools with JSON Schema input specs. Generates the `tools/list` response. |
| `src/mcp/dispatch.rs` | MCP tool dispatch. Extracts arguments from JSON params, calls the corresponding command handler, captures JSON output. |

## Key Design Decisions

### `.console.exe` Preference (Windows)

On Windows, the GUI Godot `.exe` does not write to stdout/stderr. gdcli automatically looks for a `.console.exe` sibling and uses that instead. This is handled transparently in `godot_finder.rs`.

### Pipe Deadlock Prevention

`runner.rs` spawns background threads to drain stdout and stderr **before** calling `wait_timeout()`. Without this, the process can deadlock: Godot blocks writing to a full pipe buffer, while gdcli blocks waiting for exit.

### Noise Filtering

`errors.rs` filters out non-actionable errors: anything from `res://.godot/` internal paths and "Could not load global script cache" messages that appear on every headless run.

## Environment Variables

| Variable | Purpose |
|---|---|
| `GODOT_PATH` | Path to the Godot binary. Highest priority for binary discovery. |

## Using gdcli in a Godot Project

Recommended agent workflow when editing a Godot project:

```bash
# 1. Verify environment (run once per session)
gdcli doctor

# 2. Create scripts and scenes
gdcli script create scripts/player.gd --extends CharacterBody2D --methods _ready,_physics_process
gdcli scene create scenes/player.tscn --root-type CharacterBody2D
gdcli node add scenes/player.tscn Sprite2D PlayerSprite --script res://scripts/player.gd

# 3. After editing a .gd file, lint it
gdcli script lint --file res://scripts/player.gd

# 4. After editing a .tscn file, validate it
gdcli scene validate scenes/main.tscn

# 5. Run the project to check for runtime errors
gdcli run --timeout 15

# 6. Check project structure
gdcli project info
gdcli scene list

# 7. Look up Godot API docs
gdcli docs CharacterBody2D move_and_slide

# 8. Fix stale UID refs after renaming/moving files
gdcli uid fix --dry-run   # preview
gdcli uid fix             # apply
```

Exit codes: `0` = success/clean, `1` = errors found or command failed.

## Further Reading

- `.agent/skills/` — GDScript patterns, scene structure, signals, common errors, project layout
- `.agent/workflows/godot-dev-loop.md` — step-by-step agent workflow for editing Godot projects
