# Benchmark Results

## Pilot: Main Menu Screen

| Run | Condition | Model | Score | Tokens (in) | Tokens (out) | Wall Time | Tool Calls | Notes |
|-----|-----------|-------|-------|-------------|--------------|-----------|------------|-------|
| 1   | gdcli     |       |   /20 |             |              |           |            |       |
| 2   | gdcli     |       |   /20 |             |              |           |            |       |
| 1   | godot-mcp |       |   /20 |             |              |           |            |       |
| 2   | godot-mcp |       |   /20 |             |              |           |            |       |
| 1   | bare      |       |   /20 |             |              |           |            |       |
| 2   | bare      |       |   /20 |             |              |           |            |       |

## How to fill in

### Tokens
After each run, check Claude Code's usage display or the API dashboard.
Claude Code shows token usage at the end of a session.

### Wall Time
Record start time before pasting the prompt, end time when the LLM says it's done.

### Tool Calls
Count from the conversation (Claude Code shows each tool call).

### Score
Run: `bash benchmark/score.sh benchmark/runs/<run-dir>`
