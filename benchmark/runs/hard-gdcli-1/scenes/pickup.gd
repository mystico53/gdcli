extends Area2D

signal collected

func _on_body_entered(body: Node2D) -> void:
	if body.has_method("_physics_process"):
		collected.emit()
		queue_free()
