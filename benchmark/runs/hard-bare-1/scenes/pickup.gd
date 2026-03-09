extends Area2D

signal collected

func _on_body_entered(body: Node2D) -> void:
	if body.has_method("_physics_process") and body is CharacterBody2D and body.name == "Player":
		collected.emit()
		queue_free()
