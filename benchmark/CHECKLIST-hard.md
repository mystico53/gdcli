# Scoring Checklist — Hard Benchmark

## Phase 1: Build (30 pts)

### Structure (5 pts)
- [ ] project.godot exists with main scene set
- [ ] player.tscn exists
- [ ] enemy.tscn exists
- [ ] bullet.tscn exists
- [ ] hud.tscn exists
- [ ] main.tscn exists

### Sub-resources (6 pts)
- [ ] player.tscn has sub_resource RectangleShape2D
- [ ] player.tscn CollisionShape2D references the sub_resource correctly
- [ ] enemy.tscn has sub_resource CircleShape2D
- [ ] bullet.tscn has sub_resource CircleShape2D
- [ ] hud.tscn has sub_resource StyleBoxFlat for HealthBar
- [ ] All sub_resource IDs are unique and correctly referenced

### Scene instancing (3 pts)
- [ ] main.tscn has ext_resource for player.tscn
- [ ] main.tscn instances Player with position set
- [ ] main.tscn instances HUD

### Node hierarchies (5 pts)
- [ ] player.tscn: CollisionShape2D + Sprite2D as children of root
- [ ] enemy.tscn: CollisionShape2D + Sprite2D as children of root
- [ ] hud.tscn: MarginContainer > VBoxContainer > ScoreLabel + HealthBar
- [ ] main.tscn: Enemies + Bullets containers + SpawnTimer
- [ ] Correct parent paths in all .tscn files

### Signal connections (3 pts)
- [ ] bullet.tscn: body_entered -> _on_body_entered
- [ ] main.tscn: SpawnTimer timeout -> _on_spawn_timer_timeout
- [ ] All connections reference valid node paths

### Scripts (5 pts)
- [ ] player.gd has move_and_slide physics movement
- [ ] enemy.gd chases target
- [ ] bullet.gd moves in direction + handles body_entered
- [ ] hud.gd has update_score and update_health methods
- [ ] main.gd spawns enemies on timer with edge spawning

### Runtime (3 pts)
- [ ] godot --headless --quit exits cleanly
- [ ] No SCRIPT ERROR in output
- [ ] No missing resource errors

### Phase 1 Total: /30

---

## Phase 2: Modify (15 pts)

### New scene (4 pts)
- [ ] pickup.tscn exists with Area2D root
- [ ] pickup.tscn has sub_resource CircleShape2D (radius 12)
- [ ] pickup.gd exists with body_entered + collected signal
- [ ] collision_layer=8, collision_mask=1

### main.tscn modifications (4 pts)
- [ ] PickupTimer added (wait_time=3.0, autostart=true)
- [ ] PickupTimer timeout signal connected
- [ ] Pickups container Node2D added
- [ ] Existing nodes/connections NOT broken

### main.gd modifications (3 pts)
- [ ] Preloads pickup scene
- [ ] Spawns pickups on timer
- [ ] Adds to Pickups container

### hud.tscn modifications (2 pts)
- [ ] PickupLabel added in VBoxContainer
- [ ] Existing nodes NOT broken

### hud.gd modifications (2 pts)
- [ ] update_pickups method added
- [ ] Existing methods NOT broken

### Phase 2 Total: /15

---

## Grand Total: /45

### Scoring
- 40-45: Excellent
- 30-39: Good
- 20-29: Partial
- <20: Failed
