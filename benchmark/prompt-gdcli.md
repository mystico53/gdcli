# Condition A: gdcli MCP

## Setup
1. Create a fresh directory for the run: `benchmark/runs/gdcli-N/`
2. Start Claude Code with gdcli MCP server configured, CWD set to the run directory
3. Note the start time

## System Prompt Addition
(none — gdcli tools are available via MCP as normal)

## User Prompt
Paste this exactly:

---

Build a Godot 4 project with a main menu that navigates to placeholder scenes.
Use only gdcli MCP tools for scene/node/script creation — do not write .tscn files directly.

Requirements:
- project.godot with main scene set to res://scenes/main_menu.tscn
- scenes/main_menu.tscn: Control root, ColorRect background (#1a1a2e, full rect), centered VBoxContainer with: Label "My Game" (font size 48), buttons "Play" (-> game.tscn), "Settings" (-> settings.tscn), "Quit" (-> quit). Buttons min width 200.
- scenes/game.tscn: Control root, ColorRect background (#16213e, full rect), Label "Game Scene" centered, "Back to Menu" button -> main_menu.tscn
- scenes/settings.tscn: Control root, ColorRect background (#0f3460, full rect), Label "Settings" centered, "Back to Menu" button -> main_menu.tscn
- scenes/credits.tscn: Control root, ColorRect background (#533483, full rect), Label "Credits" centered, "Back to Menu" button -> main_menu.tscn
- All transitions use get_tree().change_scene_to_file(). Quit uses get_tree().quit().
- Signal connections: button pressed signals connected to methods on root node.
- No art assets, custom fonts, or animations.

---
