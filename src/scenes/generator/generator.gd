extends Panel

const NUM_ARGS_CSV := 4
const EXPORT_DPI: float = 96.0
const ENTRY_BUTTON := preload("res://src/scenes/generator/entry_button.tscn")
const ERROR_LABEL := preload("res://src/scenes/generator/error_label.tscn")

var csv_path := ""
var entries: Array[EntryButton] = []
var selected_entry: EntryButton

@onready var file_line_edit: LineEdit = %FileLineEdit
@onready var file_pick_button: Button = %FilePickButton
@onready var sheets_container: VBoxContainer = %SheetsContainer
@onready var sheet_texture_rect: TextureRect = %SheetTextureRect


func _process(delta: float) -> void:
	# yes this is ugly but i'm too lazy to refactor this.
	if %ErrorList.get_child_count() == 0:
		%ClearButton.hide()
	else:
		%ClearButton.show()


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

	var sheet := data.create_texture(get_fase(), get_edicao(), EXPORT_DPI)
	if sheet.is_valid():
		sheet_texture_rect.texture = sheet.texture
	else:
		sheet_texture_rect.texture = null
		notify_error("(Insc. %s) - %s" % [data.inscricao, sheet.error])


func show_sheet(data: Participante):
	if data == null:
		sheet_texture_rect.texture = null
		return

	var sheet := data.create_texture(get_fase(), get_edicao(), EXPORT_DPI)
	if sheet.is_valid():
		sheet_texture_rect.texture = sheet.texture
	else:
		sheet_texture_rect.texture = null
		notify_error("(Insc. %s) - %s" % [data.inscricao, sheet.error])


func is_header(line: PackedStringArray) -> bool:
	var inscricao := line[0].to_lower() == "inscricao" or line[0].to_lower() == "inscricão" or line[0].to_lower() == "inscrição"
	var nome := line[1].to_lower() == "participante" or line[1].to_lower() == "nome"
	var escola := line[2].to_lower() == "escola"
	var modalidade := line[3].to_lower() == "modalidade" or line[3].to_lower() == "nivel"
	return inscricao and nome and escola and modalidade


func load_csv() -> void:
	show_sheet(null)
	for entry in entries:
		entry.queue_free()
	entries.clear()

	if csv_path.is_empty():
		toggle_save_button(false)
		return
	toggle_save_button(true)

	var csv_file := FileAccess.open(csv_path, FileAccess.READ)

	var l := 0
	while csv_file.get_position() < csv_file.get_length():
		l += 1

		var line := csv_file.get_csv_line()
		if len(line) != NUM_ARGS_CSV:
			notify_error("(%d) - Linha inválida: %s" % [l, ",".join(line)])
			continue

		if is_header(line):
			continue

		var inscricao := Lago.parse_inscricao(line[0])
		if inscricao == "":
			notify_error("(%d) - Número de inscrição inválido: %s" % [l, line[0]])
			continue
		var modalidade := Lago.parse_modalidade(line[3])
		if modalidade == -1:
			notify_error("(%d) - Modalidade inválida: %s" % [l, line[3]])
			continue

		var participant := Participante.create(inscricao, line[1], line[2], modalidade)
		var entry: EntryButton = ENTRY_BUTTON.instantiate()
		entry.participant = participant
		entry.entry_number = inscricao.to_int()
		entry.toggled.connect(_on_entry_toggled.bind(entry))
		sheets_container.add_child(entry)
		entries.push_back(entry)

	csv_file.close()


func toggle_save_button(toggle: bool) -> void:
	%ExportContainer.visible = toggle


func _on_file_pick_button_pressed() -> void:
	MouseBlocker.show_empty()
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
	MouseBlocker.hide()
	if new_file_path.is_empty() or new_file_path.ends_with(".csv") and FileAccess.file_exists(new_file_path):
		csv_path = new_file_path
	file_line_edit.text = csv_path # resets to old path if invalid file
	load_csv()


func _on_dir_selected(dir: String) -> void:
	var thread := Thread.new()
	var participants: Array[Participante] = []
	for e in entries:
		participants.push_back(e.participant)
	MouseBlocker.show_loading("Gerando gabaritos...\n(Isso pode demorar um pouco.)")

	var fase := get_fase()
	var edicao := get_edicao()
	var bundle: bool = %BundleCheckbox.button_pressed
	var single: bool = %SingleCheckbox.button_pressed
	var sort_schools: bool = %SchoolCheckbox.button_pressed

	var f := func() -> void: # this is nasty lol
		AnswerSheet.save_many(
			dir,
			participants,
			fase,
			edicao,
			bundle,
			single,
			sort_schools,
			func(errors: Array[String]):
				if len(errors) > 0:
					for error in errors:
						notify_error(error)
					MouseBlocker.show_dialog(
						"A geração dos gabaritos encerrou com erros! :(",
						"Ok :(",
						"",
						MouseBlocker.LagostaIcon.SAD
					)
				else:
					MouseBlocker.hide()
				thread.wait_to_finish() # this is nastier lol
		)

	thread.start(f) # nyeh heh.


func _on_file_line_edit_editing_toggled(toggled_on: bool) -> void:
	if not toggled_on:
		file_line_edit.text = csv_path # resets to old path


func _on_save_all_button_pressed() -> void:
	MouseBlocker.show_empty()
	%FileDialog.filters = []
	%FileDialog.file_mode = FileDialog.FILE_MODE_OPEN_DIR
	%FileDialog.popup_file_dialog()


func _on_close_error_button_pressed() -> void:
	MouseBlocker.hide()


func _on_options_button_toggled(toggled_on: bool) -> void:
	%OptionsContainer.visible = toggled_on


func _on_clear_button_pressed() -> void:
	for child in %ErrorList.get_children():
		child.queue_free()


func _on_file_dialog_canceled() -> void:
	MouseBlocker.hide()
