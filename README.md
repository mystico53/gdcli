# gdcli

> **Status: Archived.** This project is no longer actively maintained. See [Why archived?](#why-archived) below.

**CLI toolkit for Godot 4 — built for AI agents, works great for humans.**

gdcli gives coding agents (and you) a structured, scriptable interface to Godot projects. It parses scenes, creates scripts, validates references, and lints GDScript — all from the command line, with machine-readable JSON output by default.

- **Single binary** — one Rust executable, no runtime dependencies
- **Stock Godot 4** — works with any Godot 4 build, no patches needed
- **Filesystem-native** — scene/node/script operations parse `.tscn` files directly (instant, no Godot subprocess)
- **MCP server** — exposes all commands as tools for Claude, Cursor, Cline, and other MCP clients
- **JSON by default** — auto-detects piped output and switches to structured JSON

## Why archived?

Frontier LLMs (Claude Opus 4.6, GPT-4o, etc.) know the `.tscn` format well enough to write valid Godot scene files directly — including sub-resources, scene instancing, signal connections, and correct ext_resource/sub_resource ID management.

We benchmarked gdcli against bare LLM (no tools) on tasks ranging from simple menu screens to multi-scene 2D games with sub-resources, collision shapes, scene instancing, and a modification phase that edits existing scenes. The results:

| Task | Bare LLM | gdcli MCP |
|---|---|---|
| Simple menu (4 scenes) | 30s, 18/18 | 80s, 18/18 |
| 2D arena game (6 scenes, sub-resources, instancing) | 60s, 32/32 | 90s, 32/32 |
| Modify existing scenes (Phase 2) | 26s, 15/15 | 38s, 15/15 |

Same correctness, but bare LLM was ~50% faster every time. Both approaches produced valid scenes that ran cleanly in Godot 4.6.1 with zero errors.

gdcli still has value in specific situations:
- **Smaller models** (Haiku, older Sonnet) that don't know `.tscn` format reliably
- **Validation workflows** — `scene_validate` and `script_lint` catch errors the LLM might not notice
- **Iterative editing** of large existing scenes where reading/rewriting entire files is wasteful
- **CI/CD pipelines** where structured CLI commands are preferable to LLM-generated file content

But for the primary use case — an AI agent building Godot projects from scratch or modifying them with guidance — the tool overhead isn't justified when frontier models already know the format.

The code remains available and functional. Feel free to fork if it's useful for your workflow.

## Install

**Download binary:**

Grab the latest release from the [Releases page](https://github.com/mystico53/gdcli/releases/latest) — Windows, Linux, and macOS binaries available.

**From source:**

```sh
cargo install --git https://github.com/mystico53/gdcli
```

## Add to your AI agent

```bash
npx -y gdcli-godot setup claude-code   # Claude Code
npx -y gdcli-godot setup cursor        # Cursor
npx -y gdcli-godot setup vscode        # VS Code
```

**Manual (Claude Code):**
```bash
# macOS / Linux
claude mcp add --transport stdio gdcli -- npx -y gdcli-godot mcp

# Windows (PowerShell)
claude mcp add --transport stdio gdcli -- cmd /c npx -y gdcli-godot mcp
```

## Prerequisites

Godot 4 is required for commands that invoke the engine (`doctor`, `script lint`, `run`, `docs --build`). All other commands work without Godot installed.

```sh
export GODOT_PATH=/path/to/godot
```

Run `gdcli doctor` to verify your setup.

## Commands

### Diagnostics

| Command | Description |
|---|---|
| `gdcli doctor` | Check Godot installation and project health |

### Scripts

| Command | Description |
|---|---|
| `gdcli script lint` | Lint all GDScript files for parse/compile errors |
| `gdcli script lint --file path.gd` | Lint a single file |
| `gdcli script create path.gd --extends Node --methods _ready,_process` | Create a GDScript with boilerplate |

### Scenes

| Command | Description |
|---|---|
| `gdcli scene list` | List all `.tscn` files with node/resource counts |
| `gdcli scene validate path.tscn` | Check for broken resource references |
| `gdcli scene create path.tscn --root-type Node2D` | Create a new scene file |
| `gdcli scene edit path.tscn --set Player::speed=200` | Edit node properties in a scene |
| `gdcli scene inspect path.tscn` | Show all nodes, resources, sub-resources, and connections |
| `gdcli scene inspect path.tscn --node Player` | Inspect a single node and its referenced resources |

### Nodes

| Command | Description |
|---|---|
| `gdcli node add scene.tscn Sprite2D MySprite` | Add a node to a scene |
| `gdcli node add scene.tscn MyEnemy --instance res://scenes/enemy.tscn` | Add an instanced scene node |
| `gdcli node add scene.tscn CollisionShape2D MyShape --sub-resource RectangleShape2D` | Add a node with an inline sub-resource |
| `gdcli node remove scene.tscn MySprite` | Remove a node and its children |

### Sub-resources

| Command | Description |
|---|---|
| `gdcli sub-resource add scene.tscn RectangleShape2D --wire-node CollisionShape --wire-property shape` | Create a sub-resource and wire it to a node |
| `gdcli sub-resource edit scene.tscn RectangleShape2D_abc --set "size=Vector2(60,60)"` | Edit sub-resource properties |

### Connections

| Command | Description |
|---|---|
| `gdcli connection add scene.tscn pressed Button . _on_button_pressed` | Add a signal connection |
| `gdcli connection remove scene.tscn pressed Button . _on_button_pressed` | Remove a signal connection |

### Project

| Command | Description |
|---|---|
| `gdcli project info` | Show project metadata, autoloads, and file counts |
| `gdcli project init` | Initialize a new Godot project |

### UIDs

| Command | Description |
|---|---|
| `gdcli uid fix` | Fix stale UID references |
| `gdcli uid fix --dry-run` | Preview fixes without applying |

### Docs

| Command | Description |
|---|---|
| `gdcli docs Node2D` | Look up a Godot class |
| `gdcli docs Node2D add_child` | Look up a specific member |
| `gdcli docs --build` | Build/rebuild the docs cache |

### Runtime

| Command | Description |
|---|---|
| `gdcli run` | Run the project headlessly |
| `gdcli run --timeout 60 --scene res://levels/test.tscn` | Run a specific scene |

Non-blocking execution via MCP: `run_start` / `run_read` / `run_stop`.

## JSON output

gdcli auto-detects piped output and switches to JSON. Force it in a terminal:

```sh
gdcli doctor --json
```

```json
{
  "ok": true,
  "command": "doctor",
  "data": { ... },
  "error": null
}
```

## Architecture

| Layer | Commands | How it works |
|---|---|---|
| **Filesystem** | scene/node/script/sub-resource/connection/uid operations | Direct `.tscn` file I/O, instant, no Godot needed |
| **Godot subprocess** | doctor, script lint, run, docs build | Spawns Godot with `--headless`, captures output |

## License

[MIT](LICENSE)
