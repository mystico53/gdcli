# Benchmark: Main Menu Screen

## Goal
Build a Godot 4.x project with a main menu that navigates to three placeholder scenes.
No art assets — use ColorRect backgrounds and default fonts only.

## Exact Requirements

### Project Structure
```
project.godot
scenes/
  main_menu.tscn
  main_menu.gd
  game.tscn
  game.gd
  settings.tscn
  settings.gd
  credits.tscn
  credits.gd
```

### main_menu.tscn
- Root: Control (full rect, name: MainMenu)
- Background: ColorRect, color #1a1a2e, anchors full rect
- VBoxContainer centered on screen (anchors center)
- Title: Label, text "My Game", font size 48
- Three buttons inside VBoxContainer (minimum_size.x = 200):
  - "Play" -> transitions to game.tscn
  - "Settings" -> transitions to settings.tscn
  - "Quit" -> quits the application

### game.tscn
- Root: Control (name: Game)
- Background: ColorRect, color #16213e, anchors full rect
- Label: "Game Scene" centered
- Button: "Back to Menu" -> transitions to main_menu.tscn

### settings.tscn
- Root: Control (name: Settings)
- Background: ColorRect, color #0f3460, anchors full rect
- Label: "Settings" centered
- Button: "Back to Menu" -> transitions to main_menu.tscn

### credits.tscn
- Root: Control (name: Credits)
- Background: ColorRect, color #533483, anchors full rect
- Label: "Credits" centered
- Button: "Back to Menu" -> transitions to main_menu.tscn

### Scripts
All scene transitions use `get_tree().change_scene_to_file("res://scenes/X.tscn")`.
Quit uses `get_tree().quit()`.
Signal connections: each button's `pressed` signal connected to a method on the root node.

### project.godot
- Main scene set to `res://scenes/main_menu.tscn`
- Window size: 1152x648 (or Godot default)

## What is NOT required
- Audio, animations, transitions, or visual effects
- Custom fonts or themes
- Responsive layout beyond basic centering
- Any gameplay logic
