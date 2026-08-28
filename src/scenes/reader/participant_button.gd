@tool
class_name ParticipantButton
extends HBoxContainer

signal check_toggled(checked: bool)
signal button_toggled(toggled_on: bool)

const BUTTON_SCENE := preload("res://src/scenes/reader/participant_button.tscn")

const COLOR_WARN := Color("#EC8447")
const COLOR_ERROR := Color("#C1121F")

@export var display: SheetsContainer.SORTING:
	set = set_display
@export var file_name: String = "":
	set(f):
		file_name = f
		update_display()

var info: Participante = Participante.new():
	set(i):
		info = i
		update_display()
var answers: Array[int]

@onready var button: Button = %Button
@onready var checkbox: CheckBox = %CheckBox


static func create(_file_name: String, _info: Participante, _answers: Array[int] = []) -> ParticipantButton:
	var new_button: ParticipantButton = BUTTON_SCENE.instantiate()
	new_button.file_name = _file_name
	new_button.info = _info
	new_button.answers = _answers
	return new_button


func _ready() -> void:
	update_display()


func get_pressed() -> bool:
	return button.button_pressed


func get_checked() -> bool:
	return checkbox.button_pressed


func set_pressed(toggled_on: bool) -> void:
	button.set_pressed_no_signal(toggled_on)


func set_checked(checked: bool) -> void:
	checkbox.set_pressed_no_signal(checked)


func set_display(d: SheetsContainer.SORTING) -> void:
	display = d
	update_display()


func update_display() -> void:
	if not button:
		await ready
	match display:
		SheetsContainer.SORTING.FILE_NAME:
			button.text = file_name
		SheetsContainer.SORTING.NAME:
			button.text = info.nome
		SheetsContainer.SORTING.ID:
			button.text = info.inscricao


func _on_button_toggled(toggled_on: bool) -> void:
	button_toggled.emit(toggled_on)


func _on_check_box_toggled(toggled_on: bool) -> void:
	check_toggled.emit(toggled_on)
