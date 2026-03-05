# GDScript 4.x Patterns

Quick reference for GDScript syntax and patterns in Godot 4.x. Written for AI agents generating or editing `.gd` files.

## Script Structure

Every script must declare what it extends. Optional `class_name` registers a global type.

```gdscript
extends Node2D
class_name Player

# Constants
const SPEED: float = 200.0
const MAX_HEALTH: int = 100

# Exported variables (editable in the Inspector)
@export var health: int = 100
@export var damage: float = 10.0
@export_range(0, 100) var armor: int = 50
@export_enum("Warrior", "Mage", "Rogue") var player_class: int = 0

# Onready variables (assigned when node enters the tree)
@onready var sprite: Sprite2D = $Sprite2D
@onready var collision: CollisionShape2D = $CollisionShape2D

# Regular typed variables
var velocity: Vector2 = Vector2.ZERO
var is_alive: bool = true
var inventory: Array[String] = []
var stats: Dictionary = {}
```

## Type System

GDScript 4.x has optional static typing. Always use types for clarity.

```gdscript
# Variable declarations
var name: String = "Player"
var count: int = 0
var speed: float = 1.5
var active: bool = true
var position: Vector2 = Vector2(100, 200)
var items: Array[String] = ["sword", "shield"]
var scores: Dictionary = {"level1": 100}

# Function signatures
func take_damage(amount: int) -> void:
    health -= amount

func get_health() -> int:
    return health

func find_nearest(targets: Array[Node2D]) -> Node2D:
    # ...
    return targets[0]

# Type casting
var sprite := $Sprite2D as Sprite2D
var body := collision.get_parent() as CharacterBody2D
```

## Common Node Types and Patterns

### CharacterBody2D (player/enemy movement)

```gdscript
extends CharacterBody2D

const SPEED: float = 300.0
const JUMP_VELOCITY: float = -400.0

func _physics_process(delta: float) -> void:
    # Gravity
    if not is_on_floor():
        velocity += get_gravity() * delta

    # Jump
    if Input.is_action_just_pressed("jump") and is_on_floor():
        velocity.y = JUMP_VELOCITY

    # Horizontal movement
    var direction := Input.get_axis("move_left", "move_right")
    if direction:
        velocity.x = direction * SPEED
    else:
        velocity.x = move_toward(velocity.x, 0, SPEED)

    move_and_slide()
```

### Area2D (triggers, pickups)

```gdscript
extends Area2D

signal collected

func _ready() -> void:
    body_entered.connect(_on_body_entered)

func _on_body_entered(body: Node2D) -> void:
    if body is CharacterBody2D:
        collected.emit()
        queue_free()
```

## Annotations

| Annotation | Purpose |
|---|---|
| `@export` | Expose variable in the Inspector |
| `@export_range(min, max)` | Numeric slider in Inspector |
| `@export_enum("A", "B")` | Dropdown in Inspector |
| `@export_file("*.gd")` | File picker with filter |
| `@export_node_path` | Node path picker |
| `@onready` | Assign when node enters tree (shorthand for setting in `_ready`) |
| `@tool` | Script runs in the editor |
| `@icon("res://icon.png")` | Custom icon in editor |

## Lifecycle Callbacks

```gdscript
func _init() -> void:
    # Called when object is created (before _ready)
    pass

func _ready() -> void:
    # Called when node enters the tree (all children ready)
    pass

func _process(delta: float) -> void:
    # Called every frame
    pass

func _physics_process(delta: float) -> void:
    # Called every physics tick (fixed timestep)
    pass

func _input(event: InputEvent) -> void:
    # Called for unhandled input events
    pass

func _unhandled_input(event: InputEvent) -> void:
    # Called for input not consumed by _input or UI
    pass
```

## Lambdas and Callables

```gdscript
# Lambda syntax
var greet := func(name: String) -> String:
    return "Hello, " + name

# Callable binding
button.pressed.connect(func(): print("clicked"))

# Callable.bind() — pass extra data with signals
button.pressed.connect(_on_button.bind(button_id))

func _on_button(id: int) -> void:
    print("Button %d pressed" % id)
```

## String Formatting

```gdscript
# Format operator
var msg: String = "Player %s has %d HP" % [name, health]

# String interpolation (no built-in f-strings — use % or +)
var label: String = "Score: " + str(score)
```

## Common Built-in Methods

```gdscript
# Node tree
get_node("Child")          # or $Child
get_parent()
get_children()
add_child(node)
remove_child(node)
queue_free()               # safe deletion at end of frame

# Timers
await get_tree().create_timer(1.0).timeout

# Scene switching
get_tree().change_scene_to_file("res://scenes/game.tscn")

# Groups
add_to_group("enemies")
get_tree().get_nodes_in_group("enemies")
is_in_group("enemies")

# Input
Input.is_action_pressed("move_left")
Input.is_action_just_pressed("jump")
Input.get_axis("move_left", "move_right")
Input.get_vector("move_left", "move_right", "move_up", "move_down")
```

## Inner Classes

```gdscript
class Inventory:
    var items: Array[String] = []

    func add(item: String) -> void:
        items.append(item)

    func has(item: String) -> bool:
        return item in items
```

## Enums

```gdscript
enum State { IDLE, RUNNING, JUMPING, FALLING }

var current_state: State = State.IDLE

func update_state() -> void:
    match current_state:
        State.IDLE:
            pass
        State.RUNNING:
            pass
        State.JUMPING:
            pass
```

## Headless Script Pattern

> [!IMPORTANT]
> Scripts run via `gdcli run` or `godot -s` must `extends SceneTree` and use `_init()`. `_ready()` does not fire for SceneTree scripts.

```gdscript
extends SceneTree

func _init() -> void:
    print("Running headless")
    # Do work here
    quit()
```
