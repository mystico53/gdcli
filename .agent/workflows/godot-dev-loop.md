# Godot Development Loop with gdcli

Step-by-step workflow for AI agents editing Godot projects using gdcli for validation and testing.

## Prerequisites

- gdcli is installed and on PATH (or run via `cargo run --` from the gdcli repo)
- `GODOT_PATH` is set to a patched Godot binary with `--structured-errors` support
- Working directory is the Godot project root (contains `project.godot`)

## Workflow

### Step 1 — Verify Environment

Run once at the start of a session to confirm everything is set up:

```bash
gdcli doctor
```

Check the JSON output:
- `ok: true` means all checks passed
- If `structured_errors` check fails, the Godot binary needs the `--structured-errors` patch
- If `project_file` check fails, you're not in a Godot project directory

### Step 2 — Understand the Project

```bash
gdcli project info
gdcli scene list
```

This gives you the project name, main scene, autoloads, and a list of all scenes with node counts. Use this to orient before making changes.

### Step 3 — Edit GDScript Files

Make your changes to `.gd` files. After each edit, immediately lint:

```bash
gdcli script lint --file path/to/changed_file.gd
```

> [!IMPORTANT]
> The `--file` path should be relative to the project root (e.g., `scripts/player.gd`, not `res://scripts/player.gd`). The file must exist on disk.

**Interpreting results:**
- `ok: true` — no parse errors, safe to continue
- `ok: false` — fix the reported errors before proceeding. Errors include file path, line number, and message.

For a full project lint (checks all scripts loaded by the project):

```bash
gdcli script lint
```

### Step 4 — Edit Scene Files

After modifying `.tscn` files, validate them:

```bash
gdcli scene validate path/to/scene.tscn
```

This checks:
- All `ext_resource` paths resolve to existing files on disk
- Nodes that look like they should have types are flagged

### Step 5 — Runtime Testing

Run the project headlessly to catch runtime errors:

```bash
gdcli run --timeout 15
```

**Interpreting results:**
- `exit_code: 0` + `error_count: 0` + `timed_out: false` → clean run
- `errors` array contains any runtime errors with file, line, and message
- `timed_out: true` → the project didn't exit within the timeout. For test scripts, ensure they call `quit()`
- `stdout` contains the program's print output

Run a specific scene:

```bash
gdcli run --timeout 15 --scene res://scenes/test.tscn
```

### Step 6 — Fix UID References

After renaming or moving files, fix stale UID references:

```bash
# Preview what would change
gdcli uid fix --dry-run

# Apply fixes
gdcli uid fix
```

## Error Resolution Loop

When errors are found, follow this pattern:

```
1. Read the error (file, line, message)
2. Read the source file at that line
3. Identify and fix the issue
4. Re-lint the file: gdcli script lint --file <path>
5. If clean, run: gdcli run --timeout 15
6. Repeat until clean
```

## Exit Codes

All gdcli commands use consistent exit codes:

| Code | Meaning |
|---|---|
| `0` | Success — no errors or issues found |
| `1` | Failure — errors found, validation failed, or command error |

## JSON Output

When invoked in a non-TTY context (pipe, agent), gdcli automatically outputs JSON. Every response follows this envelope:

```json
{
  "ok": true,
  "command": "script lint",
  "data": { ... },
  "error": null
}
```

Always check `ok` first. If `false`, check `error` for a summary and `data` for details (error list, issue list, etc.).

## Quick Reference

| Task | Command |
|---|---|
| Check setup | `gdcli doctor` |
| Project overview | `gdcli project info` |
| List all scenes | `gdcli scene list` |
| Lint one file | `gdcli script lint --file <path>` |
| Lint whole project | `gdcli script lint` |
| Validate a scene | `gdcli scene validate <path>` |
| Run project | `gdcli run --timeout <secs>` |
| Run specific scene | `gdcli run --timeout <secs> --scene <res://path>` |
| Preview UID fixes | `gdcli uid fix --dry-run` |
| Apply UID fixes | `gdcli uid fix` |
