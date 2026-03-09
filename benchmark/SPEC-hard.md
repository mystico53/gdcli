# Benchmark: 2D Arena Game (Hard)

## Why this is harder
- Sub-resources (collision shapes, style boxes) with correct IDs and references
- Scene instancing with ext_resources across files
- Multiple ext_resources per scene
- Phase 2: modify existing scenes without breaking references
- Deep node hierarchies with correct parent paths

## Phase 1: Build from scratch

### Project Structure
```
project.godot
scenes/
  main.tscn           (root scene)
  main.gd
  player.tscn         (CharacterBody2D)
  player.gd
  enemy.tscn          (CharacterBody2D)
  enemy.gd
  bullet.tscn         (Area2D)
  bullet.gd
  hud.tscn            (CanvasLayer + UI)
  hud.gd
```

### player.tscn
- Root: CharacterBody2D (name: Player)
- Script: res://scenes/player.gd
- CollisionShape2D with inline sub_resource RectangleShape2D (size: Vector2(32, 32))
- Sprite2D with no texture (placeholder — just the node)
- Properties on root: collision_layer = 1, collision_mask = 2

### player.gd
```gdscript
extends CharacterBody2D

@export var speed: float = 200.0

func _physics_process(delta: float) -> void:
    var input := Vector2.ZERO
    input.x = Input.get_axis("ui_left", "ui_right")
    input.y = Input.get_axis("ui_up", "ui_down")
    velocity = input.normalized() * speed
    move_and_slide()
```

### enemy.tscn
- Root: CharacterBody2D (name: Enemy)
- Script: res://scenes/enemy.gd
- CollisionShape2D with inline sub_resource CircleShape2D (radius: 16.0)
- Sprite2D with no texture
- Properties on root: collision_layer = 2, collision_mask = 1

### enemy.gd
```gdscript
extends CharacterBody2D

@export var speed: float = 80.0
var target: Node2D = null

func _physics_process(delta: float) -> void:
    if target:
        var direction := global_position.direction_to(target.global_position)
        velocity = direction * speed
        move_and_slide()
```

### bullet.tscn
- Root: Area2D (name: Bullet)
- Script: res://scenes/bullet.gd
- CollisionShape2D with inline sub_resource CircleShape2D (radius: 4.0)
- Properties on root: collision_layer = 4, collision_mask = 2

### bullet.gd
```gdscript
extends Area2D

@export var speed: float = 400.0
var direction := Vector2.RIGHT

func _physics_process(delta: float) -> void:
    position += direction * speed * delta

func _on_body_entered(body: Node2D) -> void:
    if body.is_in_group("enemies"):
        body.queue_free()
        queue_free()
```

### bullet.tscn signal connections
- body_entered signal from "." (root) to "." method "_on_body_entered"

### hud.tscn
- Root: CanvasLayer (name: HUD)
- Script: res://scenes/hud.gd
- MarginContainer (anchors full rect, theme_override_constants: margin_left=20, margin_top=20)
  - VBoxContainer (parent: MarginContainer)
    - Label (name: ScoreLabel, parent: VBoxContainer, text: "Score: 0", theme_override_font_sizes/font_size=24)
    - ProgressBar (name: HealthBar, parent: VBoxContainer, min_value=0, max_value=100, value=100, custom_minimum_size=Vector2(200, 20))
      - with inline sub_resource StyleBoxFlat for theme_override_styles/fill (bg_color: Color(0.8, 0.1, 0.1, 1), corner_radius_top_left=4, corner_radius_top_right=4, corner_radius_bottom_left=4, corner_radius_bottom_right=4)

### hud.gd
```gdscript
extends CanvasLayer

func update_score(value: int) -> void:
    $MarginContainer/VBoxContainer/ScoreLabel.text = "Score: " + str(value)

func update_health(value: float) -> void:
    $MarginContainer/VBoxContainer/HealthBar.value = value
```

### main.tscn
- Root: Node2D (name: Main)
- Script: res://scenes/main.gd
- Instance of player.tscn (name: Player, props: position=Vector2(576, 324))
- Instance of hud.tscn (name: HUD)
- Node2D (name: Enemies) — empty container for spawned enemies
- Node2D (name: Bullets) — empty container for spawned bullets
- Timer (name: SpawnTimer, wait_time=2.0, autostart=true)
- Signal: SpawnTimer timeout -> "." method "_on_spawn_timer_timeout"

### main.gd
```gdscript
extends Node2D

var enemy_scene: PackedScene = preload("res://scenes/enemy.tscn")
var score: int = 0

func _on_spawn_timer_timeout() -> void:
    var enemy := enemy_scene.instantiate()
    var edge := randi() % 4
    var viewport_size := get_viewport_rect().size
    match edge:
        0: enemy.global_position = Vector2(randf_range(0, viewport_size.x), 0)
        1: enemy.global_position = Vector2(viewport_size.x, randf_range(0, viewport_size.y))
        2: enemy.global_position = Vector2(randf_range(0, viewport_size.x), viewport_size.y)
        3: enemy.global_position = Vector2(0, randf_range(0, viewport_size.y))
    enemy.target = $Player
    enemy.add_to_group("enemies")
    $Enemies.add_child(enemy)
```

### project.godot
- Main scene: res://scenes/main.tscn
- Window size: 1152x648

---

## Phase 2: Modify existing scenes

After Phase 1 is complete and verified, give this follow-up prompt:

> Modify the project:
> 1. Add a new scene `scenes/pickup.tscn`: Area2D root (name: Pickup), CollisionShape2D with CircleShape2D (radius: 12.0), Sprite2D (no texture). Script: scenes/pickup.gd. collision_layer=8, collision_mask=1.
> 2. Write scenes/pickup.gd: on body_entered, if body is Player, emit signal "collected" and queue_free.
> 3. In main.tscn: add a Timer named "PickupTimer" (wait_time=3.0, autostart=true), connect its timeout to "_on_pickup_timer_timeout".
> 4. In main.gd: add pickup spawning logic (similar to enemy spawning, using preload of pickup.tscn, add to a new "Pickups" Node2D).
> 5. In hud.tscn: add a Label named "PickupLabel" after HealthBar in the VBoxContainer, text "Pickups: 0".
> 6. In hud.gd: add update_pickups(value: int) method.

---

## What is NOT required
- Textures, art, audio
- Win/lose conditions
- Actual game balance
- UI polish beyond what's specified
