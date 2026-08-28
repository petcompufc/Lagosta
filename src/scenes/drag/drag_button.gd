class_name DragButton
extends TextureButton

var upper_limit_x := 0.0
var upper_limit_y := 0.0

func _process(_delta: float) -> void:
	if button_pressed:
		global_position = get_global_mouse_position() - size/2.0
		clamp_pos()


func clamp_pos() -> void:
	position.x = clampf(position.x, -size.x/2.0, upper_limit_x-size.x/2.0)
	position.y = clampf(position.y, -size.x/2.0, upper_limit_y-size.x/2.0)


func pos() -> Vector2:
	return Vector2(
		position.x + size.x/2.0,
		position.y + size.y/2.0,
	)


func global_pos() -> Vector2:
	return Vector2(
		global_position.x - size.x/2.0,
		global_position.y - size.y/2.0,
	)
