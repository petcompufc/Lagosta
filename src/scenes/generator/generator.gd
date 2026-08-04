extends Panel

const NUM_ARGS_CSV := 4
const EXPORT_SCALE := 2.5
const ENTRY_BUTTON := preload("res://src/scenes/generator/entry_button.tscn")
const ERROR_LABEL := preload("res://src/scenes/generator/error_label.tscn")

@onready var file_line_edit: LineEdit = %FileLineEdit
@onready var file_pick_button: Button = %FilePickButton
@onready var sheets_container: VBoxContainer = %SheetsContainer
@onready var sheet_texture_rect: TextureRect = %SheetTextureRect
@onready var save_button_container: VBoxContainer = $HSplitContainer/LeftContainer/SaveButtonContainer

var csv_path := ""
var entries: Array[EntryButton] = []
var selected_entry: EntryButton


func _process(delta: float) -> void:
	%LagostaLoading.offset_transform_rotation += 2.0 * delta


func get_fase() -> Lago.Fase:
	return %FaseOptionButton.selected as Lago.Fase


func get_edicao() -> String:
	return %EditionLineEdit.text


func notify_error(error: String):
	var new_label: ErrorLabel = ERROR_LABEL.instantiate()
	%ErrorList.add_child(new_label)
	new_label.text = error


func regen_sheet() -> void:
	if selected_entry == null:
		sheet_texture_rect.texture = null
		return
	var data := selected_entry.participant

	var sheet := data.to_sheet(get_fase(), get_edicao(), EXPORT_SCALE)
	if sheet.is_valid():
		sheet_texture_rect.texture = sheet.create_texture()
	else:
		sheet_texture_rect.texture = null
		notify_error("(Insc. %s) - %s" % [data.inscricao, Lago.parse_answer_sheet_error(sheet.error)])


func show_sheet(data: Participante):
	if data == null:
		sheet_texture_rect.texture = null
		return

	var sheet := data.to_sheet(get_fase(), get_edicao(), EXPORT_SCALE)
	if sheet.is_valid():
		sheet_texture_rect.texture = sheet.create_texture()
	else:
		sheet_texture_rect.texture = null
		notify_error("(Insc. %s) - %s" % [data.inscricao, Lago.parse_answer_sheet_error(sheet.error)])


func load_csv() -> void:
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

		var line := csv_file.get_csv_line()
		if len(line) != NUM_ARGS_CSV:
			notify_error("(%d) - Linha inválida: %s" % [l, ",".join(line)])
			continue
		var inscricao := Lago.parse_inscricao(line[0])
		if inscricao == "":
			notify_error("(%d) - Número de inscrição inválido: %s" % [l, line[0]])
			continue # TODO: signal error
		var modalidade := Lago.parse_modalidade(line[3])
		if modalidade == -1:
			notify_error("(%d) - Modalidade inválida: %s" % [l, line[3]])
			continue

		var participant := Participante.create(inscricao, line[1], line[2], modalidade)
		var entry: EntryButton = ENTRY_BUTTON.instantiate()
		entry.participant = participant
		entry.entry_number = l
		entry.toggled.connect(_on_entry_toggled.bind(entry))
		sheets_container.add_child(entry)
		entries.push_back(entry)

	csv_file.close()


func _on_file_pick_button_pressed() -> void:
	%MouseBlocker.show()
	%FileDialog.filters = ["*.csv"]
	%FileDialog.file_mode = FileDialog.FILE_MODE_OPEN_FILE
	%FileDialog.popup_file_dialog()


func _on_entry_toggled(toggle: bool, entry: EntryButton) -> void:
	if selected_entry:
		selected_entry.set_pressed_no_signal(false)
	if not toggle:
		selected_entry = null
		show_sheet(null)
		return
	selected_entry = entry
	show_sheet(entry.participant)


func _on_file_selected(new_file_path: String) -> void:
	hide_mouse_blocker()
	if new_file_path.is_empty() or new_file_path.ends_with(".csv") and FileAccess.file_exists(new_file_path):
		csv_path = new_file_path
	file_line_edit.text = csv_path # resets to old path if invalid file
	load_csv()


func _on_dir_selected(dir: String) -> void:
	var thread := Thread.new()
	var participants: Array[Participante] = []
	for e in entries: participants.push_back(e.participant)
	%LoadingPanel.show()

	var fase := get_fase()
	var edicao := get_edicao()
	var png: bool = %PNGCheckbox.button_pressed
	var pdf: bool = %PDFCheckbox.button_pressed

	var f := func() -> void: # this is nasty lol
		AnswerSheet.save_many(
			dir,
			participants,
			fase,
			edicao,
			EXPORT_SCALE,
			png,
			pdf,
			func(errors: Array[String]):
				%LoadingPanel.hide()
				if len(errors) > 0:
					notify_error("Geração de gabaritos encerrada com erros.")
					%FloatingErrorLabel.text = "A geração dos gabaritos encerrou com erros! :("
					%ErrorPanel.show()
				else:
					hide_mouse_blocker()
				thread.wait_to_finish() # this is nastier lol
		)

	thread.start(f) # nyeh heh.


func _on_file_line_edit_editing_toggled(toggled_on: bool) -> void:
	if not toggled_on:
		file_line_edit.text = csv_path # resets to old path


func _on_save_all_button_pressed() -> void:
	%MouseBlocker.show()
	%FileDialog.filters = []
	%FileDialog.file_mode = FileDialog.FILE_MODE_OPEN_DIR
	%FileDialog.popup_file_dialog()


func hide_mouse_blocker() -> void:
	%MouseBlocker.hide()


func _on_close_error_button_pressed() -> void:
	%ErrorPanel.hide()
	hide_mouse_blocker()
