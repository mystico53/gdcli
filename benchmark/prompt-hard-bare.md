# Hard Benchmark — Condition C: Bare LLM

## Setup
1. Create: `benchmark/runs/hard-bare-N/`
2. Claude Code with NO MCP servers, CWD = run dir
3. Note start time

## Phase 1 Prompt

---

Build a Godot 4 2D arena game project. Write all files directly (.tscn, .gd, project.godot).

Structure: project.godot + scenes/ folder with all .tscn and .gd files.
Main scene: res://scenes/main.tscn. Window size 1152x648.

**player.tscn**: CharacterBody2D root (name: Player), script res://scenes/player.gd. Children: CollisionShape2D with inline RectangleShape2D sub_resource (size Vector2(32,32)), Sprite2D (no texture). Root props: collision_layer=1, collision_mask=2.

**player.gd**: @export var speed=200.0, _physics_process with Input.get_axis("ui_left","ui_right") and ("ui_up","ui_down"), velocity=input.normalized()*speed, move_and_slide().

**enemy.tscn**: CharacterBody2D root (name: Enemy), script res://scenes/enemy.gd. Children: CollisionShape2D with inline CircleShape2D sub_resource (radius 16.0), Sprite2D. Root props: collision_layer=2, collision_mask=1.

**enemy.gd**: @export var speed=80.0, var target:Node2D=null, _physics_process chases target with direction_to, move_and_slide().

**bullet.tscn**: Area2D root (name: Bullet), script res://scenes/bullet.gd. Children: CollisionShape2D with inline CircleShape2D (radius 4.0). Root props: collision_layer=4, collision_mask=2. Signal: body_entered from root to root method "_on_body_entered".

**bullet.gd**: @export var speed=400.0, var direction=Vector2.RIGHT, _physics_process moves position+=direction*speed*delta. _on_body_entered: if body.is_in_group("enemies") then body.queue_free() and queue_free().

**hud.tscn**: CanvasLayer root (name: HUD), script res://scenes/hud.gd. Children: MarginContainer (full rect anchors, theme_override_constants: margin_left=20, margin_top=20) > VBoxContainer > ScoreLabel (Label, text "Score: 0", font_size=24) + HealthBar (ProgressBar, min=0, max=100, value=100, custom_minimum_size=Vector2(200,20), with inline StyleBoxFlat sub_resource for theme_override_styles/fill: bg_color=Color(0.8,0.1,0.1,1), all corner radii=4).

**hud.gd**: update_score(value:int) sets ScoreLabel.text, update_health(value:float) sets HealthBar.value. Use $MarginContainer/VBoxContainer/ScoreLabel paths.

**main.tscn**: Node2D root (name: Main), script res://scenes/main.gd. Instance player.tscn (name: Player, position=Vector2(576,324)). Instance hud.tscn (name: HUD). Node2D children: Enemies, Bullets. Timer: SpawnTimer (wait_time=2.0, autostart=true). Signal: SpawnTimer timeout -> root "_on_spawn_timer_timeout".

**main.gd**: preload enemy scene, var score=0. _on_spawn_timer_timeout: instantiate enemy, random edge spawn (pick edge 0-3, place on viewport boundary), set enemy.target=$Player, add to "enemies" group, add to $Enemies.

---

## Phase 2 Prompt (after Phase 1 is done)

---

Modify the project:
1. Add scenes/pickup.tscn: Area2D root (name: Pickup), CollisionShape2D with CircleShape2D (radius 12.0), Sprite2D. Script: scenes/pickup.gd. collision_layer=8, collision_mask=1.
2. Write scenes/pickup.gd: on body_entered, if body is Player (or has method), emit signal "collected" and queue_free.
3. In main.tscn: add Timer "PickupTimer" (wait_time=3.0, autostart=true), connect timeout to "_on_pickup_timer_timeout". Add Node2D "Pickups" container.
4. In main.gd: add pickup spawning (preload pickup.tscn, spawn at random position on timer, add to $Pickups).
5. In hud.tscn: add Label "PickupLabel" after HealthBar in VBoxContainer, text "Pickups: 0".
6. In hud.gd: add update_pickups(value: int) method.

---
