# gdcli — Full Build Plan
### A Rust-based agent-friendly CLI for Godot 4, by Murmur Systems

---

## What This Is

A single-binary CLI tool that wraps Godot's headless mode into clean, composable subcommands with structured JSON output — purpose-built for agentic coding loops (Claude Code, SWE-agent, CI pipelines) and developer automation.

The core problem it solves: Godot has no CLI designed for programmatic use. Raw `godot --headless` flags are cryptic, errors block on stdin, output is mixed and unparseable, and there is no reliable feedback loop for agents. `gdcli` fixes all of that.

**Tagline:** `gdcli run`, `gdcli scene list`, `gdcli script lint` — structured output, zero blocking, agent-ready.

---

## Name

**`gdcli`** — clear, short, tab-completion friendly, not taken on crates.io.

Binary name: `gdcli`
Crate name: `gdcli`
GitHub repo: `mystico53/gdcli`

---

## Why Rust

- Single binary, zero runtime — agents install it with one download, no Python env, no Node version conflicts
- Either works or doesn't — no compound failure probability from environment setup steps
- All modern terminal tooling (ripgrep, uv, Starship, Codex CLI) is Rust — fits the ecosystem aesthetic
- "Written in Rust" is a genuine marketing asset in the indie dev community
- Long-term maintenance is easier than Python — no dependency rot, no version drift

---

## Core Design Decision: Require `--structured-errors`

`gdcli` **requires** a Godot build that supports the `--structured-errors` flag. There is no timeout/kill workaround for stock Godot.

**Why:**
- The timeout/kill approach is fragile — works for immediate parse errors, breaks for runtime errors that appear mid-game
- Building for the agentic future, not for stock Godot today
- The `--structured-errors` patch is already written, the Godot proposal (#13048) is filed, and the PR is ready to submit — it will get merged
- A clean requirement is better than a silent workaround that fails unpredictably

**What `gdcli` does at startup:**
1. Find the Godot binary (PATH / `GODOT_PATH` env / common install locations)
2. Run `godot --version` to confirm it exists
3. Run `godot --headless --structured-errors --quit` as a probe
4. If exit code indicates unsupported flag: print a clear error and link to instructions for building a patched binary
5. If supported: proceed normally

This means users need a patched Godot build. That's the honest requirement. The README will explain it clearly with a one-command build script.

---

## Phase 0 — Verify Before Building (Do This First)

**Do not write any Rust until this test passes.** Everything downstream depends on the patch working correctly on Windows.

### 0.1 Install build dependencies (Windows)

```powershell
# Python + SCons (Godot's build system)
winget install Python.Python.3
pip install scons

# Visual Studio Build Tools with C++ workload
# Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/
# Select: "Desktop development with C++"
```

### 0.2 Build patched Godot

```powershell
# Clone Godot source
git clone https://github.com/godotengine/godot.git
cd godot

# Apply the structured-errors patch
git apply C:\path\to\structured-errors-final.patch

# Build headless template only — much faster than full editor
# (~15-20 min on a modern machine)
scons platform=windows target=template_debug debug_symbols=no

# Output: bin\godot.windows.template_debug.x86_64.exe
```

### 0.3 Test the patch

Create a test script with a deliberate error:

```gdscript
# test_error.gd
func _ready():
    var x = null
    x.velocity = 5
```

Run it:

```powershell
.\bin\godot.windows.template_debug.x86_64.exe `
    --headless --structured-errors -s test_error.gd

# Expected output:
# ERROR res://test_error.gd:3: Invalid set index 'velocity' on base 'Null instance'.

# Check exit code — must be non-zero
echo $LASTEXITCODE
```

Also test clean case (no errors):

```powershell
# Expected: process exits 0, no ERROR lines
.\bin\godot.windows.template_debug.x86_64.exe `
    --headless --structured-errors -s clean_script.gd
echo $LASTEXITCODE   # must be 0
```

**If both tests pass: proceed to Phase 1.**
**If not: debug the patch before touching Rust.**

### 0.4 Submit the upstream PR

In parallel with building `gdcli`, submit the patch as a PR to `godotengine/godot`. The patch is already written and clean. This costs ~30 minutes and benefits everyone permanently.

The PR description and git workflow are already prepared in `PR-GUIDE.md`.

---

## Phase 1 — Rust Project Bootstrap (Week 1)

### 1.1 Create the project

```bash
cargo new gdcli
cd gdcli
```

**`Cargo.toml`:**
```toml
[package]
name = "gdcli"
version = "0.1.0"
edition = "2021"
description = "Agent-friendly CLI for Godot 4"
license = "MIT"

[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
which = "6"
colored = "2"
wait-timeout = "0.2"   # cross-platform process timeout
```

### 1.2 Godot runner (`src/runner.rs`)

The foundation everything else builds on. All commands flow through this.

```rust
pub struct RunResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

pub fn run(args: &[&str], timeout_secs: u64) -> Result<RunResult>
```

Key responsibilities:
- Find Godot binary: check `GODOT_PATH` env → PATH → common Windows locations (`C:\Godot\`, `%APPDATA%\Godot\`)
- Always pass `--structured-errors` — it's a hard requirement
- Timeout via `wait-timeout` crate (cross-platform, works on Windows)
- Capture stdout + stderr separately
- Return clean `RunResult` — no panics, errors as `anyhow::Result`

### 1.3 Startup probe (`src/godot_finder.rs`)

Run once at startup, cache the result:

```rust
pub struct GodotInfo {
    pub path: PathBuf,
    pub version: String,
    pub structured_errors_supported: bool,
}

pub fn find_and_probe() -> Result<GodotInfo>
```

If `structured_errors_supported` is false, print:

```
Error: your Godot build does not support --structured-errors.

gdcli requires a patched Godot build. To build one:
  https://github.com/mystico53/gdcli#building-godot

Set GODOT_PATH to point to your patched binary:
  set GODOT_PATH=C:\path\to\godot.windows.template_debug.x86_64.exe
```

### 1.4 JSON output envelope (`src/output.rs`)

All commands emit this in `--json` mode:

```json
{
  "ok": true,
  "exit_code": 0,
  "command": "script lint",
  "output": "...",
  "errors": [],
  "duration_ms": 342
}
```

In TTY mode (human reading), format with colors. In non-TTY / `--json` mode, emit the envelope. Agents run in non-TTY contexts so they get JSON automatically without needing to pass `--json`.

---

## Phase 2 — Core Commands (Weeks 2–3)

### `gdcli doctor`

The `flutter doctor` equivalent. First command any user runs.

```
gdcli doctor

✓ Godot 4.4 found at C:\Godot\godot.exe
✓ --structured-errors supported
✓ project.godot found in current directory
✗ Export templates not installed
✓ 47 GDScript files parsed, 0 errors
```

Exits non-zero if any required check fails.

### `gdcli script lint`

Parse-check all `.gd` files without running the project. Uses Godot's `--check-only` / `--parse` mode with `--structured-errors`.

```powershell
gdcli script lint
# Scans all .gd files, emits structured errors, exits non-zero if any found

gdcli script lint --file src/player.gd
# Lint single file

gdcli script lint --json
# Machine-readable output
```

**This is the most valuable command for the agentic loop.** Claude writes code → `gdcli script lint` → clean error lines → Claude fixes → repeat. No game window, no scene loading, pure syntax feedback in seconds.

### `gdcli run`

Run the project headlessly and capture all output.

```powershell
gdcli run --timeout 30
gdcli run --timeout 30 --json
```

Streams stdout in real-time. Exits cleanly when the scene exits or timeout is reached. With `--structured-errors`, any runtime error prints a clean `ERROR file:line: message` line and the process continues rather than hanging.

### `gdcli project info`

JSON dump of the project — parsed from `project.godot`.

```json
{
  "name": "MyGame",
  "godot_version": "4.4",
  "main_scene": "res://scenes/main.tscn",
  "autoloads": ["GameState", "AudioManager"],
  "script_count": 47,
  "scene_count": 23
}
```

---

## Phase 3 — Scene & Node Commands (Week 4)

### `gdcli scene list`

List all `.tscn` files with node counts. Parsed directly from `.tscn` text format — no Godot invocation needed, instant.

### `gdcli scene validate <path>`

Check a scene for broken references: missing scripts, missing resources, stale UIDs. Pure filesystem checks.

### `gdcli node add <scene> <type> <name>`

Add a node to a scene by directly editing the `.tscn` text format.

```powershell
gdcli node add scenes/main.tscn RigidBody2D "Enemy"
```

### `gdcli uid fix`

Scan and fix stale UID references after Godot 4.4+ file moves or renames.

```powershell
gdcli uid fix --dry-run   # show what would change
gdcli uid fix             # apply fixes
```

---

## Phase 4 — Agent Skills Layer (Week 5)

### `AGENTS.md`

Root-level file that Claude Code reads automatically. Describes the recommended agentic workflow:

```markdown
## Recommended workflow

1. Before making changes: `gdcli doctor`
2. After editing .gd files: `gdcli script lint`
3. After editing scenes: `gdcli scene validate`
4. To test runtime: `gdcli run --timeout 15`
5. Exit codes: 0 = success, non-zero = errors. Always check.
6. Add --json for machine-readable output on any command.

## Error format
ERROR res://src/player.gd:15: Invalid get index 'velocity' on base 'Nil'.
```

### `.agent/skills/`

```
.agent/skills/
├── gdscript-patterns.md     # GDScript 4.x typed syntax, common patterns
├── scene-structure.md       # How .tscn files work, node naming
├── signal-patterns.md       # Signal-driven architecture
├── common-errors.md         # Error message → likely cause → fix
└── project-layout.md        # Recommended project structure
```

`common-errors.md` is the key differentiator — a curated map of the most frequent Godot error messages to their likely root causes and fixes. When Claude sees `Invalid get index 'velocity' on base 'Null instance'`, it can reference this skill to understand it's a null reference issue and look for where the node isn't ready yet.

---

## Phase 5 — Distribution (Week 6)

### Cross-platform builds via GitHub Actions

Develop on Windows. CI cross-compiles for all three platforms automatically on every tagged release:

```yaml
# Targets:
# - windows x86_64 (.exe)
# - linux x86_64 (musl static — runs on any distro)
# - macOS x86_64 + ARM (universal binary)
```

Use the `cross` crate for Linux/macOS cross-compilation from Windows. Static musl binary for Linux means no glibc version issues — runs everywhere.

### Install

**Windows (PowerShell):**
```powershell
winget install gdcli
# or
irm https://raw.githubusercontent.com/mystico53/gdcli/main/install.ps1 | iex
```

**Linux/macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/mystico53/gdcli/main/install.sh | sh
```

### README structure

- What problem it solves (one sentence)
- A GIF of the agentic loop: edit GDScript → `gdcli script lint` → error → fix → clean
- Install command
- "Written in Rust. Single binary. No dependencies."
- Brief note on `--structured-errors` requirement + link to build instructions

### Positioning vs godot-mcp

| | godot-mcp | gdcli |
|---|---|---|
| Interface | MCP server | CLI / shell |
| Context cost | ~50k tokens | ~0 (just `--help`) |
| Dependencies | Node.js | None (single binary) |
| Composability | MCP calls only | Shell pipes, scripts, makefiles |
| Agent support | Claude Desktop / Cursor | Any agent with shell access |
| Error blocking | Yes (stock Godot) | No (`--structured-errors`) |

Not a replacement — a different philosophy. Some users will want both.

---

## The Agentic Loop

```
Claude writes/edits GDScript
         ↓
gdcli script lint
         ↓
  ┌──────┴──────┐
  │             │
exit 0        exit 1
  │             │
Continue    Read ERROR lines
            Claude fixes
                ↓
         gdcli script lint
                ↓
             exit 0
                ↓
      gdcli run --timeout 15
                ↓
      Read runtime output
      Claude iterates
```

No human copy-paste. No hanging processes. No cryptic raw Godot output. The agent drives the full loop autonomously.

---

## File Structure

```
gdcli/
├── src/
│   ├── main.rs              # CLI entry point, clap setup
│   ├── commands/
│   │   ├── doctor.rs
│   │   ├── project.rs
│   │   ├── script.rs
│   │   ├── scene.rs
│   │   ├── node.rs
│   │   ├── run.rs
│   │   └── uid.rs
│   ├── runner.rs            # Godot subprocess wrapper
│   ├── godot_finder.rs      # Binary detection + probe
│   ├── scene_parser.rs      # .tscn / .tres parser
│   └── output.rs            # JSON envelope, terminal formatting
├── .agent/
│   └── skills/
│       ├── gdscript-patterns.md
│       ├── common-errors.md
│       └── project-layout.md
├── AGENTS.md
├── Cargo.toml
├── README.md
└── .github/
    └── workflows/
        └── release.yml
```

---

## Timeline

| Week | Focus | Done when |
|---|---|---|
| 0 | Patch verified | `--structured-errors` works on Windows, exits 0/1 correctly |
| 1 | Rust bootstrap | `gdcli doctor` runs and finds Godot |
| 2 | Core commands | `script lint` and `run` work end-to-end |
| 3 | Polish | JSON output, `--help` text, error messages |
| 4 | Scene/node | `scene list/validate`, `node add`, `uid fix` |
| 5 | Agent layer | `AGENTS.md`, `.agent/skills/`, `common-errors.md` |
| 6 | Ship | GitHub Actions CI, release binaries, README, launch |

---

## Order of Operations (Start Here)

1. Apply `structured-errors-final.patch` to Godot source on Windows
2. Build `godot.windows.template_debug.x86_64.exe`
3. Run the two test cases (error case + clean case) — verify exit codes
4. If tests pass: `cargo new gdcli`
5. Submit the upstream Godot PR in parallel (30 min, already written)
6. Build the runner layer first — everything else depends on it
