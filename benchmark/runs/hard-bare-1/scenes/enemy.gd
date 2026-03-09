extends CharacterBody2D

@export var speed: float = 80.0
var target: Node2D = null

func _physics_process(_delta: float) -> void:
	if target:
		var direction := global_position.direction_to(target.global_position)
		velocity = direction * speed
		move_and_slide()
