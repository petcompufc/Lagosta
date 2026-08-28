class_name SheetsContainer
extends VBoxContainer

signal participant_checked(participant: ParticipantButton, checked: bool)
signal participant_clicked(participant: ParticipantButton, toggled_on: bool)

@export var order_crescent: bool = true: set = set_order_crescent
@onready var check_all_button: CheckBox = %CheckAllButton
@onready var sort_option_button: OptionButton = %SortOptionButton
@onready var sort_button: Button = %SortButton
@onready var sheets_container: VBoxContainer = %SheetsContainer

var current_selected: ParticipantButton

const ARROW_UP: Texture2D = preload("res://assets/icons/ui/arrow-badge-up.svg")
const ARROW_DOWN: Texture2D = preload("res://assets/icons/ui/arrow-badge-down.svg")

enum SORTING {
	FILE_NAME = 0,
	NAME = 1,
	ID = 2,
}

func _ready() -> void:
	for i in range(10):
		add_sheet("File %d" % i, Participante.create(str(i), "Nome %d" % i, "escola", 0))
	set_order_crescent(order_crescent)


func get_checked() -> Array[ParticipantButton]:
	var checked: Array[ParticipantButton] = []
	for sheet: ParticipantButton in sheets_container.get_children():
		if sheet.get_checked():
			checked.push_back(sheet)
	return checked


func set_order_crescent(toggle: bool = not order_crescent) -> void:
	order_crescent = toggle
	if order_crescent:
		sort_button.icon = ARROW_DOWN
	else:
		sort_button.icon = ARROW_UP
	update_sorting()


func update_sorting() -> void:
	var sheets := sheets_container.get_children()
	var order := 1 if order_crescent else -1
	match sort_option_button.selected:
		SORTING.FILE_NAME:
			sheets.sort_custom(func(a: ParticipantButton, b: ParticipantButton):
				return a.file_name.naturalnocasecmp_to(b.file_name) * order < 0)
		SORTING.NAME:
			sheets.sort_custom(func(a: ParticipantButton, b: ParticipantButton):
				return a.info.nome.naturalnocasecmp_to(b.info.nome) * order < 0)
		SORTING.ID:
			sheets.sort_custom(func(a: ParticipantButton, b: ParticipantButton):
				return a.info.inscricao.casecmp_to(b.info.inscricao) * order < 0)
	for i in range(len(sheets)):
		var sheet: ParticipantButton = sheets[i]
		sheet.display = sort_option_button.selected as SORTING
		sheets_container.move_child(sheet, i)


func add_sheet(file_name: String, participant: Participante, answers: Array[int] = []) -> void:
	var participant_button := ParticipantButton.create(file_name, participant, answers)
	participant_button.check_toggled.connect(_on_participant_button_checked.bind(participant_button))
	participant_button.button_toggled.connect(_on_participant_button_clicked.bind(participant_button))
	sheets_container.add_child(participant_button)


func clear_sheets() -> void:
	for child in sheets_container.get_children():
		child.queue_free()


func _on_sort_option_button_item_selected(_index: int) -> void:
	update_sorting()


func _on_check_all_button_toggled(toggled_on: bool) -> void:
	for sheet: ParticipantButton in sheets_container.get_children():
		sheet.set_checked(toggled_on)


func _on_participant_button_checked(checked: bool, participant_button: ParticipantButton) -> void:
	if checked and sheets_container.get_children().all(func(p: ParticipantButton): return p.get_checked()):
		check_all_button.set_pressed_no_signal(true)
	else:
		check_all_button.set_pressed_no_signal(false)
	participant_checked.emit(participant_button, checked)


func _on_participant_button_clicked(toggled_on: bool, participant_button: ParticipantButton) -> void:
	if current_selected and current_selected != participant_button:
		current_selected.set_pressed(false)
	current_selected = participant_button if toggled_on else null
	participant_clicked.emit(participant_button, toggled_on)
