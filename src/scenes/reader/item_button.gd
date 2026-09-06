@tool
class_name ItemButton
extends HBoxContainer

signal item_selected(answer: Lago.Answer)
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
		option_button.select(option_button.get_item_index(selected_answer))


func _on_option_button_item_selected(index: int) -> void:
	selected_answer = option_button.get_item_id(index)
	item_selected.emit(selected_answer as Lago.Answer)


func set_answer(a: int) -> void:
	selected_answer = a
