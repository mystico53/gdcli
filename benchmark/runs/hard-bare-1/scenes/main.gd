extends Node2D

var enemy_scene: PackedScene = preload("res://scenes/enemy.tscn")
var pickup_scene: PackedScene = preload("res://scenes/pickup.tscn")
var score: int = 0

func _on_spawn_timer_timeout() -> void:
	var enemy := enemy_scene.instantiate()
	var viewport_size := get_viewport_rect().size
	var edge := randi() % 4
	match edge:
		0: # Top
			enemy.position = Vector2(randf_range(0, viewport_size.x), 0)
		1: # Right
			enemy.position = Vector2(viewport_size.x, randf_range(0, viewport_size.y))
		2: # Bottom
			enemy.position = Vector2(randf_range(0, viewport_size.x), viewport_size.y)
		3: # Left
			enemy.position = Vector2(0, randf_range(0, viewport_size.y))
	enemy.target = $Player
	enemy.add_to_group("enemies")
	$Enemies.add_child(enemy)

func _on_pickup_timer_timeout() -> void:
	var pickup := pickup_scene.instantiate()
	var viewport_size := get_viewport_rect().size
	pickup.position = Vector2(randf_range(50, viewport_size.x - 50), randf_range(50, viewport_size.y - 50))
	$Pickups.add_child(pickup)
