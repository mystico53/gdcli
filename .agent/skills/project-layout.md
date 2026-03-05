# Godot Project Layout

Directory structure and organization for Godot 4 projects.

> [!NOTE]
> Godot does not enforce any directory structure — pick a convention and stay consistent across the project.

## Layout Patterns

### Feature-based (recommended)

Group related files together by area. This is the [officially recommended approach](https://docs.godotengine.org/en/stable/tutorials/best_practices/project_organization.html) and scales well as projects grow.

```
project.godot
├── characters/
│   ├── player/
│   │   ├── player.tscn
│   │   ├── player.gd
│   │   └── player_sprite.png
│   └── enemy/
│       ├── enemy.tscn
│       ├── enemy.gd
│       └── enemy_sprite.png
├── levels/
│   ├── level_01/
│   │   ├── level_01.tscn
│   │   └── level_01_tileset.tres
│   └── level_02/
│       └── level_02.tscn
├── ui/
│   ├── main_menu/
│   │   ├── main_menu.tscn
│   │   ├── main_menu.gd
│   │   └── background.png
│   ├── hud/
│   │   ├── hud.tscn
│   │   └── hud.gd
│   └── pause_menu/
│       ├── pause_menu.tscn
│       └── pause_menu.gd
├── shared/
│   ├── autoload/
│   │   ├── game_manager.gd
│   │   └── events.gd
│   ├── components/
│   │   ├── health_component.gd
│   │   └── hitbox_component.gd
│   └── themes/
│       └── default_theme.tres
└── addons/                    # Third-party plugins
```

### Type-based (alternative for smaller projects)

Separate files by type. Simpler to set up but scenes and scripts become disconnected as the project grows.

```
project.godot
├── scenes/
│   ├── main.tscn
│   ├── ui/
│   │   ├── main_menu.tscn
│   │   └── hud.tscn
│   ├── levels/
│   │   └── level_01.tscn
│   └── characters/
│       ├── player.tscn
│       └── enemy.tscn
├── scripts/
│   ├── player.gd
│   ├── enemy.gd
│   ├── autoload/
│   │   ├── game_manager.gd
│   │   └── events.gd
│   └── components/
│       ├── health_component.gd
│       └── hitbox_component.gd
├── assets/
│   ├── sprites/
│   ├── audio/
│   │   ├── sfx/
│   │   └── music/
│   └── fonts/
├── resources/
│   ├── themes/
│   └── data/
└── addons/
```

> [!IMPORTANT]
> File paths in Godot are case-sensitive on Linux and macOS even though Windows ignores case. Use consistent **lowercase** for all file and folder names to avoid export issues on other platforms.

## Key Files

### `project.godot`

INI-like format. Key sections:

```ini
[application]
config/name="My Game"
run/main_scene="res://ui/main_menu/main_menu.tscn"
config/features=PackedStringArray("4.4")

[autoload]
GameManager="*res://shared/autoload/game_manager.gd"
Events="*res://shared/autoload/events.gd"

[input]
move_left={
"deadzone": 0.5,
"events": [Object(InputEventKey,"resource_local_to_scene":false,"resource_name":"","device":-1,"window_id":0,"alt_pressed":false,"shift_pressed":false,"ctrl_pressed":false,"meta_pressed":false,"pressed":false,"keycode":65,"physical_keycode":0,"key_label":0,"unicode":97,"location":0,"echo":false,"script":null)]
}
```

### `.godot/` (auto-generated)

```
.godot/
├── imported/              # Imported asset cache
├── editor/                # Editor state
├── uid_cache.bin          # UID → path binary cache
└── global_script_class_cache.cfg
```

> [!IMPORTANT]
> Never edit files inside `.godot/` manually. This directory is auto-generated and should be in `.gitignore`.

### `.gdignore`

Placing an empty `.gdignore` file in a directory prevents Godot from importing its contents. Useful for:
- Raw source assets (PSD, Blender files) that shouldn't be imported
- Build artifacts and output directories
- Test fixtures or scratch files

## Autoload Pattern

Autoloads are singletons loaded before any scene. Registered in `project.godot` under `[autoload]`.

```ini
[autoload]
GameManager="*res://shared/autoload/game_manager.gd"
```

The `*` prefix means the script runs as a standalone node (most common). Access from anywhere:

```gdscript
GameManager.start_game()
```

Common autoloads:
- **GameManager** — game state, score, level transitions
- **Events** — global signal bus
- **SaveLoad** — save/load game data
- **AudioManager** — sound effect and music playback

## Naming Conventions

| Item | Convention | Example |
|---|---|---|
| Scene files | `snake_case.tscn` | `main_menu.tscn` |
| Script files | `snake_case.gd` | `player_controller.gd` |
| Node names | `PascalCase` | `PlayerSprite`, `HealthBar` |
| Variables | `snake_case` | `max_health`, `move_speed` |
| Constants | `UPPER_SNAKE_CASE` | `MAX_SPEED`, `GRAVITY` |
| Signals | `snake_case` (past tense) | `health_changed`, `died` |
| Functions | `snake_case` | `take_damage()`, `get_health()` |
| Classes | `PascalCase` | `class_name PlayerStats` |
| Enums | `PascalCase` type, `UPPER_SNAKE_CASE` values | `enum State { IDLE, RUNNING }` |

## Scene + Script Pairing

Two common patterns:

### 1. Co-located (recommended)

Scene and script live in the same directory:

```
characters/player/player.tscn
characters/player/player.gd
```

### 2. Type-separated

Scene and script in parallel directory trees:

```
scenes/characters/player.tscn
scripts/characters/player.gd
```

> [!NOTE]
> Co-location is the officially recommended approach. It keeps related files together and makes it easier to move or refactor features as a unit.

## Resource Files (.tres)

Custom data stored as text resources:

```
[gd_resource type="Resource" script_class="WeaponData" load_steps=2 format=3]

[ext_resource type="Script" path="res://shared/data/weapon_data.gd" id="1"]

[resource]
script = ExtResource("1")
weapon_name = "Sword"
damage = 25
attack_speed = 1.2
```

Use for data-driven design (item stats, level config, dialogue trees).

## .gitignore

```gitignore
# Godot auto-generated
.godot/

# OS
.DS_Store
Thumbs.db

# IDE
.vscode/
*.swp

# Exports
*.pck
*.zip
export/
```
