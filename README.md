# gdcli

**CLI toolkit for Godot 4 — built for AI agents, works great for humans.**

gdcli gives coding agents (and you) a structured, scriptable interface to Godot projects. It parses scenes, creates scripts, validates references, and lints GDScript — all from the command line, with machine-readable JSON output by default.

- **Single binary** — one Rust executable, no runtime dependencies
- **Stock Godot 4** — works with any Godot 4 build, no patches needed
- **Filesystem-native** — scene/node/script operations parse `.tscn` files directly (instant, no Godot subprocess)
- **MCP server** — exposes all commands as tools for Claude, Cursor, Cline, and other MCP clients
- **JSON by default** — auto-detects piped output and switches to structured JSON

### How it compares

Most Godot AI tooling (like godot-mcp) requires a Node.js runtime and runs everything through a Godot subprocess. gdcli takes a different approach:

| | gdcli | godot-mcp |
|---|---|---|
| Runtime | Single Rust binary | Node.js + npm |
| Interface | CLI + MCP server | MCP only |
| Scene operations | Direct `.tscn` parsing (instant) | Godot subprocess |
| Godot requirement | Only for lint/run/docs | Always |

## Install

**From source (recommended):**

```sh
cargo install --git https://github.com/mystico53/gdcli
```

**Linux / macOS install script:**

```sh
curl -fsSL https://raw.githubusercontent.com/mystico53/gdcli/main/install.sh | sh
```

**Windows (PowerShell):**

```powershell
irm https://raw.githubusercontent.com/mystico53/gdcli/main/install.ps1 | iex
```

## Prerequisites

Godot 4 is required for commands that invoke the engine (`doctor`, `script lint`, `run`, `docs --build`). All other commands work without Godot installed.

If Godot isn't on your PATH, set `GODOT_PATH`:

```sh
export GODOT_PATH=/path/to/godot
```

On Windows, point to the `.console.exe` variant for proper stdout capture:

```powershell
$env:GODOT_PATH = "C:\path\to\Godot_v4.x-stable_win64_console.exe"
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

### Nodes

| Command | Description |
|---|---|
| `gdcli node add scene.tscn Sprite2D MySprite --parent Player --script res://scripts/my.gd --props texture=icon.png` | Add a node to a scene |
| `gdcli node remove scene.tscn MySprite` | Remove a node and its children |

### Project

| Command | Description |
|---|---|
| `gdcli project info` | Show project metadata, autoloads, and file counts |

### UIDs

| Command | Description |
|---|---|
| `gdcli uid fix` | Fix stale UID references in `.tscn`/`.tres` files |
| `gdcli uid fix --dry-run` | Preview fixes without applying |

### Docs

| Command | Description |
|---|---|
| `gdcli docs Node2D` | Look up a Godot class |
| `gdcli docs Node2D add_child` | Look up a specific member |
| `gdcli docs Node2D --members` | List all methods, properties, and signals |
| `gdcli docs --build` | Build/rebuild the docs cache (runs `godot --doctool`) |

### Runtime

| Command | Description |
|---|---|
| `gdcli run` | Run the project headlessly (30s default timeout) |
| `gdcli run --timeout 60 --scene res://levels/test.tscn` | Run a specific scene with custom timeout |

### MCP Server

| Command | Description |
|---|---|
| `gdcli mcp` | Start MCP server (JSON-RPC 2.0 over stdio) |
| `gdcli mcp --project-dir /path/to/project` | Start with explicit project directory |

## JSON output

gdcli auto-detects when stdout is piped or redirected and switches to JSON. Agents get structured data by default — no flags needed.

Force JSON in a terminal:

```sh
gdcli doctor --json
```

Every response uses the same envelope:

```json
{
  "ok": true,
  "command": "doctor",
  "data": { ... },
  "error": null
}
```

- `ok` — `true` if the command succeeded with no issues, `false` otherwise
- `command` — the subcommand name
- `data` — command-specific payload
- `error` — human-readable error string, or `null`

## MCP server mode

gdcli includes a built-in [MCP](https://modelcontextprotocol.io/) server that exposes all 14 commands as tools. Any MCP-compatible client (Claude Code, Claude Desktop, Cursor, Cline, etc.) can use it.

**Configure in `.mcp.json`** (or your client's MCP config):

```json
{
  "mcpServers": {
    "gdcli": {
      "command": "gdcli",
      "args": ["mcp"],
      "cwd": "/path/to/your/godot/project"
    }
  }
}
```

The `cwd` should point to your Godot project root (the directory with `project.godot`). Alternatively, use the `--project-dir` flag:

```json
{
  "mcpServers": {
    "gdcli": {
      "command": "gdcli",
      "args": ["mcp", "--project-dir", "/path/to/your/godot/project"]
    }
  }
}
```

All 14 tools are exposed: `doctor`, `project_info`, `scene_list`, `scene_validate`, `scene_create`, `scene_edit`, `node_add`, `node_remove`, `uid_fix`, `script_create`, `script_lint`, `run`, `docs`, `docs_build`.

## Agentic workflow

```
  Agent                     gdcli                    Godot
    |                         |                        |
    |-- gdcli doctor -------->|--- probe version ----->|
    |<-- JSON: ok, version ---|                        |
    |                         |                        |
    |-- gdcli script create ->|                        |
    |<-- JSON: file created --|  (filesystem only)     |
    |                         |                        |
    |-- gdcli scene create -->|                        |
    |<-- JSON: scene created -|  (filesystem only)     |
    |                         |                        |
    |-- gdcli node add ------>|                        |
    |<-- JSON: node added ----|  (filesystem only)     |
    |                         |                        |
    |-- gdcli script lint --->|--- --check-only ------>|
    |<-- JSON: errors[] ------|                        |
    |                         |                        |
    |   (fix errors in .gd)   |                        |
    |                         |                        |
    |-- gdcli script lint --->|--- --check-only ------>|
    |<-- JSON: ok, 0 errors --|                        |
    |                         |                        |
    |-- gdcli run ----------->|--- headless run ------>|
    |<-- JSON: exit 0 --------|                        |
```

## Architecture

gdcli commands split into two layers:

| Layer | Commands | How it works |
|---|---|---|
| **Filesystem** | `project info`, `scene list/validate/create/edit`, `node add/remove`, `uid fix`, `script create`, `docs` | Direct file I/O — parses `.tscn`, `.tres`, `.uid`, `.godot`, and XML files. Instant, no Godot needed. |
| **Godot subprocess** | `doctor`, `script lint`, `run`, `docs --build` | Spawns Godot with `--headless`, captures stdout/stderr. Works with stock Godot 4. |

## Contributing

Contributions welcome! gdcli is early-stage and there's plenty to do:

- New commands and features
- Better error messages
- Platform-specific fixes (Linux/macOS testing especially welcome)
- Documentation and examples

```sh
git clone https://github.com/mystico53/gdcli
cd gdcli
cargo build
cargo test
```

See [AGENTS.md](AGENTS.md) for architecture details and source layout.

## License

[MIT](LICENSE)
