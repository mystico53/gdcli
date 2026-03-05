# Godot Signal Patterns

Reference for signal-driven architecture in Godot 4. Signals are Godot's observer pattern implementation — they decouple emitters from receivers.

## Declaring Custom Signals

```gdscript
# No parameters
signal died

# With parameters
signal health_changed(old_value: int, new_value: int)
signal item_collected(item_name: String, quantity: int)
signal game_state_changed(new_state: int)
```

## Emitting Signals

```gdscript
func take_damage(amount: int) -> void:
    var old_health := health
    health -= amount
    health_changed.emit(old_health, health)
    if health <= 0:
        died.emit()
```

## Connecting Signals in Code

```gdscript
func _ready() -> void:
    # Connect to own signal
    health_changed.connect(_on_health_changed)

    # Connect to child node's signal
    $Button.pressed.connect(_on_button_pressed)

    # Connect with lambda
    $Timer.timeout.connect(func(): print("Timer fired"))

    # Connect with bound arguments
    $Button.pressed.connect(_on_button.bind("start"))

    # One-shot connection (auto-disconnects after first emit)
    $Timer.timeout.connect(_on_timeout, CONNECT_ONE_SHOT)

func _on_health_changed(old_value: int, new_value: int) -> void:
    print("Health: %d -> %d" % [old_value, new_value])

func _on_button_pressed() -> void:
    print("Button pressed")

func _on_button(button_name: String) -> void:
    print("Button: " + button_name)
```

## Connecting Signals in the Editor

Signals connected via the Godot editor appear in the `.tscn` file as `[connection]` entries:

```
[connection signal="pressed" from="StartButton" to="." method="_on_start_pressed"]
```

> [!NOTE]
> Editor-connected signals use the naming convention `_on_NodeName_signal_name` or `_on_signal_name`. Code-connected signals can use any method name.

## Disconnecting Signals

```gdscript
# Disconnect a specific connection
health_changed.disconnect(_on_health_changed)

# Check if connected before disconnecting
if health_changed.is_connected(_on_health_changed):
    health_changed.disconnect(_on_health_changed)
```

## Common Built-in Signals

### UI Signals

| Node | Signal | Fires when |
|---|---|---|
| `Button` | `pressed` | Button clicked |
| `Button` | `toggled(toggled_on: bool)` | Toggle button state changes |
| `LineEdit` | `text_submitted(text: String)` | Enter pressed |
| `LineEdit` | `text_changed(new_text: String)` | Text modified |
| `TextEdit` | `text_changed` | Text modified |
| `ItemList` | `item_selected(index: int)` | Item clicked |
| `OptionButton` | `item_selected(index: int)` | Option chosen |

### Physics Signals

| Node | Signal | Fires when |
|---|---|---|
| `Area2D/3D` | `body_entered(body: Node)` | Physics body enters |
| `Area2D/3D` | `body_exited(body: Node)` | Physics body exits |
| `Area2D/3D` | `area_entered(area: Area)` | Another area enters |
| `Area2D/3D` | `area_exited(area: Area)` | Another area exits |

### Lifecycle Signals

| Node | Signal | Fires when |
|---|---|---|
| `Node` | `ready` | Node and children entered tree |
| `Node` | `tree_entered` | Node added to tree |
| `Node` | `tree_exiting` | Node about to leave tree |
| `CanvasItem` | `visibility_changed` | Visibility toggled |

### Animation Signals

| Node | Signal | Fires when |
|---|---|---|
| `AnimationPlayer` | `animation_finished(name: StringName)` | Animation ends |
| `AnimationPlayer` | `animation_started(name: StringName)` | Animation begins |
| `Timer` | `timeout` | Timer reaches 0 |
| `Tween` | `finished` | Tween completes |

## Signal Architecture Patterns

### Event Bus (Global Signals)

Use an autoload singleton for game-wide events:

```gdscript
# events.gd (registered as autoload "Events")
extends Node

signal score_changed(new_score: int)
signal player_died
signal level_completed(level_id: int)
```

```gdscript
# Any script can emit
Events.score_changed.emit(new_score)

# Any script can listen
func _ready() -> void:
    Events.player_died.connect(_on_player_died)
```

### Upward Communication (Child → Parent)

Children emit signals, parents connect to them. Children never reference parents directly.

```gdscript
# health_component.gd
extends Node
signal health_depleted

func take_damage(amount: int) -> void:
    health -= amount
    if health <= 0:
        health_depleted.emit()
```

```gdscript
# enemy.gd
extends CharacterBody2D

@onready var health_comp: Node = $HealthComponent

func _ready() -> void:
    health_comp.health_depleted.connect(_on_health_depleted)

func _on_health_depleted() -> void:
    queue_free()
```

### Await Signals

```gdscript
# Wait for a signal before continuing
func show_dialog(text: String) -> void:
    $DialogBox.text = text
    $DialogBox.show()
    await $DialogBox.confirmed  # pauses until signal emits
    $DialogBox.hide()

# Wait for timer
await get_tree().create_timer(2.0).timeout
print("2 seconds passed")
```

## Common Mistakes

| Mistake | Fix |
|---|---|
| Connecting a signal twice → `Signal already connected` error | Check `is_connected()` before connecting, or use `CONNECT_ONE_SHOT` |
| Connecting in `_process` → connects every frame | Connect in `_ready()` or `_init()` |
| Signal receiver freed → errors on emit | Disconnect in `_exit_tree()` or use `CONNECT_ONE_SHOT` |
| Wrong parameter count in handler | Handler signature must match signal declaration |
