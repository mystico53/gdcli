# Godot Scene (.tscn) File Structure

Reference for the `.tscn` text scene format used by Godot 4. Useful for agents reading, generating, or modifying scene files directly.

## File Format Overview

`.tscn` files use Godot's text resource format. Sections are delimited by `[brackets]` and properties are `key = value` pairs. Order matters: header first, then ext_resources, sub_resources, nodes, connections.

## Header

```
[gd_scene load_steps=4 format=3 uid="uid://cg3hylang5fxn"]
```

| Attribute | Meaning |
|---|---|
| `load_steps` | Total number of resources + 1 (used for loading progress) |
| `format` | Scene format version (`3` for Godot 4.x) |
| `uid` | Unique identifier for the scene (`uid://...` format). Godot 4.4+ generates these automatically. |

## External Resources (`ext_resource`)

References to files outside this scene (scripts, textures, other scenes, etc.):

```
[ext_resource type="Script" uid="uid://bv6y7in6otgcm" path="res://scripts/player.gd" id="1_j0gfq"]
[ext_resource type="Texture2D" uid="uid://d2k4m8n1p3q5r" path="res://assets/player.png" id="2_abc12"]
[ext_resource type="PackedScene" uid="uid://x9y8z7w6v5u4t" path="res://scenes/bullet.tscn" id="3_def34"]
```

| Attribute | Meaning |
|---|---|
| `type` | Resource type (`Script`, `Texture2D`, `PackedScene`, `AudioStream`, etc.) |
| `uid` | Stable unique ID. Survives file renames — Godot resolves resources by UID first, path second. |
| `path` | `res://` path to the resource file |
| `id` | Local ID used to reference this resource within the scene (e.g., `"1_j0gfq"`) |

Referenced in node properties as `ExtResource("1_j0gfq")`.

## Sub-Resources (`sub_resource`)

Resources defined inline within the scene (not in separate files):

```
[sub_resource type="RectangleShape2D" id="RectangleShape2D_abc"]
size = Vector2(32, 64)

[sub_resource type="Animation" id="Animation_xyz"]
resource_name = "walk"
length = 0.8
loop_mode = 1
```

Referenced in node properties as `SubResource("RectangleShape2D_abc")`.

## Nodes (`node`)

```
[node name="Player" type="CharacterBody2D"]
script = ExtResource("1_j0gfq")
position = Vector2(100, 200)

[node name="Sprite" type="Sprite2D" parent="."]
texture = ExtResource("2_abc12")

[node name="CollisionShape" type="CollisionShape2D" parent="."]
shape = SubResource("RectangleShape2D_abc")

[node name="Camera" type="Camera2D" parent="."]
zoom = Vector2(2, 2)
```

| Attribute | Meaning |
|---|---|
| `name` | Node name (must be unique among siblings) |
| `type` | Node class (`Node2D`, `Sprite2D`, `CharacterBody2D`, etc.) |
| `parent` | Path relative to root node. Omitted for the root node itself. `.` = direct child of root. `Child/Grandchild` = nested path. |

### Parent Path Conventions

| Parent value | Meaning |
|---|---|
| *(omitted)* | This is the root node of the scene |
| `.` | Direct child of root |
| `Player` | Child of a node named "Player" (which is a child of root) |
| `Player/Sprite` | Child of Sprite, which is child of Player |

### Instanced Scenes

When a node is an instance of another scene, it has no `type` but has an `instance` attribute:

```
[node name="Enemy" parent="." instance=ExtResource("3_def34")]
position = Vector2(400, 300)
```

> [!NOTE]
> Instanced nodes have no `type` attribute. gdcli's scene validator flags these as warnings ("Node has no type — may be an instanced scene").

## Connections (`connection`)

Signal connections between nodes:

```
[connection signal="pressed" from="StartButton" to="." method="_on_start_pressed"]
[connection signal="body_entered" from="HitBox" to="." method="_on_hit"]
```

| Attribute | Meaning |
|---|---|
| `signal` | Signal name |
| `from` | Node path (relative to root) that emits the signal |
| `to` | Node path that receives the signal |
| `method` | Method name called on the receiver |

## UID System

Godot 4.4+ assigns a stable `uid://...` to every resource. UIDs survive file renames — when a file is moved, Godot updates the path but keeps the UID. The UID→path mapping is stored in:

- **`.uid` sidecar files** — e.g., `player.gd.uid` contains the UID for `player.gd`
- **`.godot/uid_cache.bin`** — binary cache (not human-readable)

When a file is renamed but `.tscn`/`.tres` references still have the old `path=`, the UID can be used to resolve the correct new path. This is what `gdcli uid fix` does.

## Complete Example

```
[gd_scene load_steps=3 format=3 uid="uid://cg3hylang5fxn"]

[ext_resource type="Script" uid="uid://bv6y7in6otgcm" path="res://main.gd" id="1_j0gfq"]
[ext_resource type="Texture2D" uid="uid://d2k4m8n1p3q5r" path="res://icon.svg" id="2_abc12"]

[sub_resource type="RectangleShape2D" id="RectangleShape2D_xyz"]
size = Vector2(64, 64)

[node name="Main" type="Node2D"]
script = ExtResource("1_j0gfq")

[node name="Sprite" type="Sprite2D" parent="."]
texture = ExtResource("2_abc12")

[node name="Area" type="Area2D" parent="."]

[node name="CollisionShape" type="CollisionShape2D" parent="Area"]
shape = SubResource("RectangleShape2D_xyz")

[connection signal="body_entered" from="Area" to="." method="_on_area_body_entered"]
```
