@tool
class_name ItemButton
extends HBoxContainer

@onready var label: Label = %Label
@onready var option_button: OptionButton = %OptionButton

@export var item_number: int = 1:
	set(i):
		item_number = i
		if not label:
			await ready
		label.text = "%02d:" % item_number
		option_button.tooltip_text = "Resposta da questão %d" % item_number

var selected_answer: int = 0:
	set(a):
		selected_answer = clampi(a, 0, 5)
		if not option_button:
			await ready
		option_button.selected = selected_answer

func _on_option_button_item_selected(index: int) -> void:
	selected_answer = index


func set_answer(a: int) -> void:
	selected_answer = a
