extends CanvasLayer

func update_score(value: int) -> void:
	$MarginContainer/VBoxContainer/ScoreLabel.text = "Score: " + str(value)

func update_health(value: float) -> void:
	$MarginContainer/VBoxContainer/HealthBar.value = value

func update_pickups(value: int) -> void:
	$MarginContainer/VBoxContainer/PickupLabel.text = "Pickups: " + str(value)
