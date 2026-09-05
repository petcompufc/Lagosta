extends Button

const eye_line: Texture2D = preload("res://assets/icons/ui/eye.svg")
const eye_fill: Texture2D = preload("res://assets/icons/ui/eye_fill.svg")

func _ready() -> void:
	_on_toggled(button_pressed)

func _on_toggled(toggled_on: bool) -> void:
	if toggled_on:
		icon = eye_fill
	else:
		icon = eye_line
