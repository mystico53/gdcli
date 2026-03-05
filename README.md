# gdcli

Agent-friendly CLI for Godot 4.

gdcli gives AI coding agents (and humans) a structured interface to Godot projects. Instead of parsing Godot's verbose editor output, agents get clean JSON with exit codes they can act on.

## Install

**Linux / macOS:**

```sh
curl -fsSL https://raw.githubusercontent.com/mystico53/gdcli/main/install.sh | sh
```

**Windows (PowerShell):**

```powershell
irm https://raw.githubusercontent.com/mystico53/gdcli/main/install.ps1 | iex
```

**From source:**

```sh
cargo install --git https://github.com/mystico53/gdcli
```

## Prerequisites

gdcli requires a patched Godot 4 build with `--structured-errors` support. Set the `GODOT_PATH` environment variable to point to your patched binary:

```sh
export GODOT_PATH=/path/to/patched/godot
```

On Windows:

```powershell
$env:GODOT_PATH = "C:\path\to\patched\godot.console.exe"
```

Run `gdcli doctor` to verify your setup.

## Commands

| Command | Description |
|---|---|
| `gdcli doctor` | Check Godot installation and project health |
| `gdcli script lint` | Lint all GDScript files for parse errors |
| `gdcli script lint --file path.gd` | Lint a single GDScript file |
| `gdcli run` | Run the project headlessly (default 30s timeout) |
| `gdcli run --timeout 60 --scene res://level.tscn` | Run a specific scene with custom timeout |
| `gdcli project info` | Show project metadata and file counts |
| `gdcli scene list` | List all scenes with node/resource counts |
| `gdcli scene validate path.tscn` | Check a scene for broken resource references |
| `gdcli uid fix` | Fix stale UID references in scene/resource files |
| `gdcli uid fix --dry-run` | Preview UID fixes without applying |

## JSON output

gdcli auto-detects when stdout is not a terminal (piped or redirected) and switches to JSON output. This means agents get structured data by default without any extra flags.

Force JSON in a terminal with `--json`:

```sh
gdcli doctor --json
```

Every response follows this envelope:

```json
{
  "ok": true,
  "command": "doctor",
  "data": { ... },
  "error": null
}
```

## Agentic loop

```
  Agent                     gdcli                    Godot
    |                         |                        |
    |-- gdcli doctor -------->|                        |
    |<-- JSON: ok, version ---|--- probe structured ---|
    |                         |                        |
    |-- gdcli script lint --->|                        |
    |<-- JSON: errors[] ------|--- --check-only -------|
    |                         |                        |
    |   (fix errors in .gd)   |                        |
    |                         |                        |
    |-- gdcli script lint --->|                        |
    |<-- JSON: ok, 0 errors --|--- --check-only -------|
    |                         |                        |
    |-- gdcli run ----------->|                        |
    |<-- JSON: exit 0 --------|--- headless run -------|
    |                         |                        |
```

## License

MIT
