@tool
class_name ErrorLabel
extends HBoxContainer

signal button_pressed

@export var text := "" :
	set(t):
		text = t
		if not %Label:
			await ready
		%Label.text = t
		tooltip_text = t


func _on_button_pressed() -> void:
	button_pressed.emit()
	queue_free()
