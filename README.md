# gdcli

**CLI toolkit for Godot 4 — built for AI agents, works great for humans.**

gdcli gives coding agents (and you) a structured, scriptable interface to Godot projects. It parses scenes, creates scripts, validates references, and lints GDScript — all from the command line, with machine-readable JSON output by default.

- **Single binary** — one Rust executable, no runtime dependencies
- **Stock Godot 4** — works with any Godot 4 build, no patches needed
- **Filesystem-native** — scene/node/script operations parse `.tscn` files directly (instant, no Godot subprocess)
- **MCP server** — exposes all commands as tools for Claude, Cursor, Cline, and other MCP clients
- **JSON by default** — auto-detects piped output and switches to structured JSON

## Add to your AI agent

**Claude Code (one command, no install needed):**
```bash
claude mcp add --transport stdio gdcli -- npx -y gdcli-godot mcp
```

**Cursor / VS Code / other MCP clients** — add to your MCP config:
```json
{ "mcpServers": { "gdcli": { "command": "npx", "args": ["-y", "gdcli-godot", "mcp"] } } }
```

**Using native binary (faster startup, requires [Install](#install) first):**
```bash
claude mcp add --transport stdio gdcli -- gdcli mcp
```

### Why gdcli over godot-mcp

Other Godot MCP tools launch the Godot engine for every operation — adding a node, changing a property, wiring a texture all wait for the engine to start up. gdcli reads and writes `.tscn` files directly, so those same operations take milliseconds instead of seconds.

| | gdcli | godot-mcp |
|---|---|---|
| Runtime | Single Rust binary | Node.js + npm |
| Interface | CLI + MCP server | MCP only |
| Scene edits | Direct `.tscn` parsing (~1ms) | Godot subprocess (~2s) |
| Godot required | Only for lint/run/docs | Always |
| Non-blocking run | `run_start` / `run_read` / `run_stop` | Blocks server until done |
| Offline use | 19 of 23 tools work without Godot | Nothing works without Godot |

**Concrete advantages:**

- **Speed** — adding a node, editing a property, or wiring a sub-resource takes milliseconds. Agents iterate faster when the edit-lint-run loop isn't bottlenecked by engine startup.
- **No runtime dependencies** — `cargo install` or download the binary. No `npm install`, no `node_modules`, no version conflicts.
- **CLI + MCP in one binary** — every MCP tool is also a CLI command. Agents can use MCP; humans can use the shell; CI can use either. Same tool, same output format.
- **Non-blocking project execution** — `run_start` spawns Godot in the background and returns immediately. The agent can continue editing files, then poll with `run_read` or stop with `run_stop`. godot-mcp's run blocks the entire server.
- **Works without Godot installed** — scene creation, node manipulation, sub-resources, connections, UID fixes, project info, docs lookup — none of these need Godot on the machine. Only lint, run, doctor, and docs build require the engine.

## Install

**Download binary (fastest startup):**

Grab the latest release for your platform from the [Releases page](https://github.com/mystico53/gdcli/releases/latest) — Windows, Linux, and macOS (universal) binaries available. Extract and add to your PATH.

**From source:**

```sh
cargo install --git https://github.com/mystico53/gdcli
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
| `gdcli scene inspect path.tscn` | Show all nodes, resources, sub-resources, and connections |
| `gdcli scene inspect path.tscn --node Player` | Inspect a single node and its referenced resources |

### Nodes

| Command | Description |
|---|---|
| `gdcli node add scene.tscn Sprite2D MySprite --parent Player --script res://scripts/my.gd --props texture=icon.png` | Add a node to a scene |
| `gdcli node add scene.tscn MyEnemy --instance res://scenes/enemy.tscn` | Add an instanced scene node |
| `gdcli node add scene.tscn CollisionShape2D MyShape --sub-resource RectangleShape2D --sub-resource-props "size=Vector2(30,30)"` | Add a node with an inline sub-resource |
| `gdcli node remove scene.tscn MySprite` | Remove a node and its children |

### Sub-resources

| Command | Description |
|---|---|
| `gdcli sub-resource add scene.tscn RectangleShape2D --props "size=Vector2(40,40)" --wire-node CollisionShape --wire-property shape` | Create a sub-resource and wire it to a node |
| `gdcli sub-resource add scene.tscn CircleShape2D` | Create an unwired sub-resource (emits a warning) |
| `gdcli sub-resource edit scene.tscn RectangleShape2D_abc --set "size=Vector2(60,60)"` | Edit properties on an existing sub-resource |

### Sprites

| Command | Description |
|---|---|
| `gdcli load-sprite scene.tscn MySprite res://icon.svg` | Add a Sprite2D with a texture in one call |
| `gdcli load-sprite scene.tscn MySprite res://icon.svg --sprite-type Sprite3D --parent Player` | Sprite3D under a specific parent |
| `gdcli load-sprite scene.tscn MySprite res://icon.svg --props "position=Vector2(100,200)"` | With additional properties |

### Connections

| Command | Description |
|---|---|
| `gdcli connection add scene.tscn pressed Button . _on_button_pressed` | Add a signal connection between nodes |
| `gdcli connection remove scene.tscn pressed Button . _on_button_pressed` | Remove a signal connection |

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
| `gdcli run` | Run the project headlessly (30s default timeout, blocks until done) |
| `gdcli run --timeout 60 --scene res://levels/test.tscn` | Run a specific scene with custom timeout |

**Streaming sessions (MCP):** For non-blocking execution, use `run_start` / `run_read` / `run_stop` via MCP. The agent starts Godot in the background, continues editing files, then polls for output — no server blocking.

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

gdcli includes a built-in [MCP](https://modelcontextprotocol.io/) server that exposes all 23 commands as tools. Any MCP-compatible client (Claude Code, Claude Desktop, Cursor, Cline, etc.) can use it.

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

All 23 tools are exposed: `doctor`, `project_info`, `scene_list`, `scene_validate`, `scene_create`, `scene_edit`, `scene_inspect`, `node_add`, `node_remove`, `load_sprite`, `sub_resource_add`, `sub_resource_edit`, `connection_add`, `connection_remove`, `uid_fix`, `script_create`, `script_lint`, `run`, `run_start`, `run_read`, `run_stop`, `docs`, `docs_build`.

## Agentic workflow

```
  Agent                     gdcli                    Godot
    |                         |                        |
    |-- doctor -------------->|--- probe version ----->|
    |<-- JSON: ok, version ---|                        |
    |                         |                        |
    |-- script create ------->|                        |
    |<-- JSON: file created --|  (filesystem only)     |
    |                         |                        |
    |-- scene create -------->|                        |
    |<-- JSON: scene created -|  (filesystem only)     |
    |                         |                        |
    |-- load_sprite --------->|                        |
    |<-- JSON: sprite added --|  (filesystem only)     |
    |                         |                        |
    |-- node add ------------>|                        |
    |<-- JSON: node added ----|  (filesystem only)     |
    |                         |                        |
    |-- script lint --------->|--- --check-only ------>|
    |<-- JSON: errors[] ------|                        |
    |                         |                        |
    |   (fix errors in .gd)   |                        |
    |                         |                        |
    |-- run_start ----------->|--- headless run ------>|
    |<-- JSON: session_id ----|       (background)     |
    |                         |                        |
    |   (continue editing)    |          ...           |
    |                         |                        |
    |-- run_read ------------>|                        |
    |<-- JSON: new output ----|                        |
    |                         |                        |
    |-- run_stop ------------>|--- kill if running --->|
    |<-- JSON: all output ----|                        |
```

## Architecture

gdcli commands split into two layers:

| Layer | Commands | How it works |
|---|---|---|
| **Filesystem** | `project info`, `scene list/validate/create/edit/inspect`, `node add/remove`, `load-sprite`, `sub-resource add/edit`, `connection add/remove`, `uid fix`, `script create`, `docs` | Direct file I/O — parses `.tscn`, `.tres`, `.uid`, `.godot`, and XML files. Instant, no Godot needed. |
| **Godot subprocess** | `doctor`, `script lint`, `run`, `run_start/read/stop`, `docs --build` | Spawns Godot with `--headless`, captures stdout/stderr. Works with stock Godot 4. |

## Contributing

gdcli is built with AI tools, for AI tools. Contributions using AI agents, copilots, or any other workflow are welcome — there's no disclosure requirement. What matters is the outcome.

### Getting started

```sh
git clone https://github.com/mystico53/gdcli
cd gdcli
cargo build
cargo test
```

### Quality gates

Every PR should pass these before review:

- `cargo build` compiles cleanly
- `cargo test` — all tests pass
- `cargo clippy` — no warnings
- Actually test the feature/fix with a real Godot project

### Areas of contribution

- New commands and features
- Better error messages
- Platform-specific fixes (Linux/macOS testing especially welcome)
- Documentation and examples

### For AI agents

Start with [AGENTS.md](AGENTS.md) — it covers architecture, source layout, and design decisions. [CLAUDE.md](CLAUDE.md) has project-specific instructions for Claude Code.

### PR tips

- Keep PRs focused — one feature or fix per PR
- Describe what changed and why
- Mention what you tested (which Godot version, which OS, what scenario)

## License

[MIT](LICENSE)
