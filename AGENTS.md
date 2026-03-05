# gdcli — Agent Developer Guide

Agent-friendly CLI for Godot 4. Provides structured, machine-readable output for linting GDScript, running projects headlessly, validating scenes, and fixing UID references.

## Project Overview

gdcli wraps a **patched Godot 4 binary** that supports `--structured-errors`, a custom flag that outputs errors in a parseable `ERROR res://file.gd:LINE: message` format. The CLI discovers the Godot binary dynamically (env var → PATH → common install paths) and probes its capabilities at runtime.

All commands emit a JSON envelope `{ ok, command, data, error }` when stdout is not a TTY or when `--json` is passed. This makes every command directly consumable by AI agents without extra parsing.

> [!IMPORTANT]
> gdcli requires a patched Godot build with `--structured-errors` support. Set `GODOT_PATH` to point to the patched binary. The standard Godot download will not work for commands that invoke Godot (`doctor`, `script lint`, `run`).

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
cargo run -- uid fix --dry-run
```

## Architecture

gdcli has a two-layer design:

| Layer | Commands | How it works |
|---|---|---|
| **Filesystem** | `project info`, `scene list`, `scene validate`, `uid fix` | Pure file I/O — parses `.godot`, `.tscn`, `.tres`, `.uid` files directly. Instant, no Godot needed. |
| **Godot subprocess** | `doctor`, `script lint`, `run` | Spawns the Godot binary with `--headless` and captures stdout/stderr. Requires patched Godot. |

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
| `src/runner.rs` | Spawns Godot as a subprocess. `run()` adds `--headless --structured-errors`; `run_raw()` runs with exact args. Drains stdout/stderr in background threads to prevent pipe deadlocks. Handles timeouts via `wait-timeout`. |
| `src/godot_finder.rs` | Discovers Godot binary (`GODOT_PATH` → `which` → common paths). Probes `--version` and `--structured-errors` support. Prefers `.console.exe` on Windows. |
| `src/output.rs` | `JsonEnvelope<T>` struct, `use_json()` TTY detection, `emit_json()`, colored TTY helpers (`print_check`, `print_header`, `print_error`). |
| `src/errors.rs` | Parses Godot error output. `parse_errors()` handles structured `ERROR res://` lines. `parse_script_errors()` handles `SCRIPT ERROR:` blocks from `--check-only`. Filters noise errors (`.godot/` internal, global script cache). |
| `src/scene_parser.rs` | Parses `.tscn` files into `ParsedScene` (header, ext_resources, sub_resources, nodes, connections). Also provides `find_scene_files()` for recursive `.tscn` discovery. |
| `src/commands/mod.rs` | Module declarations for all command handlers. |
| `src/commands/doctor.rs` | `gdcli doctor` — checks Godot binary, `--structured-errors` support, `project.godot` presence, `.gd` file count. |
| `src/commands/script.rs` | `gdcli script lint` — single-file lint via `--check-only` (no structured errors), project-wide lint via `--structured-errors --quit`. |
| `src/commands/run.rs` | `gdcli run` — runs project headlessly with timeout. Reports exit code, stdout, stderr, parsed errors, timeout status. |
| `src/commands/project.rs` | `gdcli project info` — parses `project.godot` for name, main scene, autoloads, script/scene counts. |
| `src/commands/scene.rs` | `gdcli scene list` / `gdcli scene validate` — lists scenes with node counts, validates ext_resource paths exist on disk. |
| `src/commands/uid.rs` | `gdcli uid fix` — scans `.uid` files to build UID→path map, finds stale path refs in `.tscn`/`.tres`, fixes them. `--dry-run` supported. |

## Key Design Decisions

### `.console.exe` Preference (Windows)

On Windows, the GUI Godot `.exe` does not write to stdout/stderr. gdcli automatically looks for a `.console.exe` sibling and uses that instead. This is handled transparently in `godot_finder.rs`.

### Pipe Deadlock Prevention

`runner.rs` spawns background threads to drain stdout and stderr **before** calling `wait_timeout()`. Without this, the process can deadlock: Godot blocks writing to a full pipe buffer, while gdcli blocks waiting for exit.

### `--check-only` Without `--structured-errors`

Single-file linting uses `--check-only` which requires Godot to exit after checking. But `--structured-errors` implies `-d` (debug mode) which prevents exit. So single-file lint uses `run_raw()` (no structured errors) and parses `SCRIPT ERROR:` blocks from stderr instead.

### Timeout-as-Success Probing

When probing `--structured-errors` support, gdcli runs `--structured-errors --version`. Because `--structured-errors` implies `-d`, Godot may not exit on its own. A timeout with version output on stdout is treated as "flag supported."

### Noise Filtering

`errors.rs` filters out non-actionable errors: anything from `res://.godot/` internal paths and "Could not load global script cache" messages that appear on every headless run.

## Environment Variables

| Variable | Purpose |
|---|---|
| `GODOT_PATH` | Path to the patched Godot binary. Highest priority for binary discovery. |

## Using gdcli in a Godot Project

Recommended agent workflow when editing a Godot project:

```bash
# 1. Verify environment (run once per session)
gdcli doctor

# 2. After editing a .gd file, lint it
gdcli script lint --file res://scripts/player.gd

# 3. After editing a .tscn file, validate it
gdcli scene validate scenes/main.tscn

# 4. Run the project to check for runtime errors
gdcli run --timeout 15

# 5. Check project structure
gdcli project info
gdcli scene list

# 6. Fix stale UID refs after renaming/moving files
gdcli uid fix --dry-run   # preview
gdcli uid fix             # apply
```

Exit codes: `0` = success/clean, `1` = errors found or command failed.

## Further Reading

- `.agent/skills/` — GDScript patterns, scene structure, signals, common errors, project layout
- `.agent/workflows/godot-dev-loop.md` — step-by-step agent workflow for editing Godot projects
