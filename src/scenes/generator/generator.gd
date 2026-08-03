extends Panel

const NUM_ARGS_CSV := 4
const ENTRY_BUTTON := preload("res://src/scenes/generator/entry_button.tscn")

@onready var file_line_edit: LineEdit = %FileLineEdit
@onready var file_pick_button: Button = %FilePickButton
@onready var sheets_container: VBoxContainer = %SheetsContainer
@onready var sheet_texture_rect: TextureRect = %SheetTextureRect
@onready var save_button_container: VBoxContainer = $HSplitContainer/LeftContainer/SaveButtonContainer

var csv_path := ""

var fase := Lago.Fase.FASE_1
var edicao := ""
var export_scale := 4.0

var entries: Array[EntryButton] = []
var selected_entry: EntryButton

func _on_file_pick_button_pressed() -> void:
	%FileDialog.popup_file_dialog()


func _load_csv() -> void:
	show_sheet(null)
	for entry in entries:
		entry.queue_free()
	entries.clear()
	
	if csv_path.is_empty():
		save_button_container.hide()
		return
	save_button_container.show()

	var csv_file := FileAccess.open(csv_path, FileAccess.READ)

	csv_file.get_csv_line() # ignore header line
	var l := 0
	while csv_file.get_position() < csv_file.get_length():
		l += 1
		
		var line = csv_file.get_csv_line()
		if len(line) != NUM_ARGS_CSV:
			continue # TODO: signal error
		var modalidade := Lago.parse_modalidade(line[3])
		if modalidade == -1:
			continue # TODO: signal error
		
		var participant := Participante.new(line[0], line[1], line[2], modalidade)
		var entry: EntryButton = ENTRY_BUTTON.instantiate()
		entry.participant = participant
		entry.entry_number = l
		entry.toggled.connect(_on_entry_toggled.bind(entry))
		sheets_container.add_child(entry)
		entries.push_back(entry)
	
	csv_file.close()


func _on_entry_toggled(toggle: bool, entry: EntryButton) -> void:
	if selected_entry:
		selected_entry.set_pressed_no_signal(false)
	if not toggle:
		selected_entry = null
		show_sheet(null)
		return
	selected_entry = entry
	show_sheet(entry.participant)


func show_sheet(data: Participante):
	if data == null:
		sheet_texture_rect.texture = null
		return
	var sheet := data.to_sheet(fase, edicao, export_scale)
	sheet_texture_rect.texture = sheet.create_texture()


func _on_file_selected(new_file_path: String) -> void:
	if new_file_path.is_empty() or new_file_path.ends_with(".csv") and FileAccess.file_exists(new_file_path):
		csv_path = new_file_path
	file_line_edit.text = csv_path # resets to old path if invalid file
	_load_csv()


func _on_file_line_edit_editing_toggled(toggled_on: bool) -> void:
	if not toggled_on:
		file_line_edit.text = csv_path # resets to old path
