class_name BetterMenuButton
extends MenuButton

signal id_pressed(id: int)

func _ready() -> void:
	get_popup().id_pressed.connect(id_pressed.emit)
