extends Panel

const NUM_ARGS_PARTICIPANTES_CSV := 4
const NUM_ARGS_ANSWERS_CSV := 6

var directory_path: String = ""
var db_path: String = ""
var answers_path: String = ""
var participant_db: Dictionary[int, Participante] = {}
var answer_table: AnswerTable = null

@onready var folder_line_edit: LineEdit = %FolderLineEdit
@onready var folder_dialog: FileDialog = %FolderDialog
@onready var db_file_dialog: FileDialog = %DBFileDialog
@onready var answers_file_dialog: FileDialog = %AnswersFileDialog
@onready var db_line_edit: LineEdit = %DBLineEdit
@onready var answers_line_edit: LineEdit = %AnswersLineEdit
@onready var db_warning_texture: TextureRect = %DBWarningTexture
@onready var answers_warning_texture: TextureRect = %AnswersWarningTexture
@onready var reading_h_split: HSplitContainer = %ReadingHSplit
@onready var info_panel: InfoPanel = %InfoPanel
@onready var sheet_preview_texture: TextureRect = %SheetPreviewTextureRect
@onready var drag_rect: DragRect = %DragRect
@onready var sheets_container: SheetsContainer = %SheetsContainer
@onready var gamma_spin_box: SpinBox = %GammaSpinBox
@onready var threshold_spin_box: SpinBox = %ThresholdSpinBox


func _ready() -> void:
	reading_h_split.hide()


static func is_header(line: PackedStringArray) -> bool:
	var inscricao := line[0].to_lower() == "inscricao" or line[0].to_lower() == "inscricão" or line[0].to_lower() == "inscrição"
	var nome := line[1].to_lower() == "participante" or line[1].to_lower() == "nome"
	var escola := line[2].to_lower() == "escola"
	var modalidade := line[3].to_lower() == "modalidade" or line[3].to_lower() == "nivel"
	return inscricao and nome and escola and modalidade


static func is_answer_header(line: PackedStringArray) -> bool:
	var a := line[0].to_lower() == "questão ini_a"
	var b := line[1].to_lower() == "peso"
	var c := line[2].to_lower() == "questão ini_b"
	var d := line[3].to_lower() == "peso"
	var e := line[4].to_lower() == "questão prog"
	var f := line[5].to_lower() == "peso"
	return a and b and c and d and e and f


func get_answer_table(csv_path: String) -> AnswerTable:
	if csv_path.is_empty():
		return null

	var csv_file := FileAccess.open(csv_path, FileAccess.READ)
	var l := 0
	var ini_a: Array[Dictionary] = []
	var ini_b: Array[Dictionary] = []
	var prog: Array[Dictionary] = []
	while csv_file.get_position() < csv_file.get_length():
		l += 1

		var line := csv_file.get_csv_line()
		if len(line) != NUM_ARGS_ANSWERS_CSV:
			_on_answers_file_selected("")
			popup_error("(Linha %d) - Linha inválida: %s" % [l, ",".join(line)])
			return null
		
		if is_answer_header(line):
			continue
		
		var p1 := line[1].replace(",", ".")
		var p2 := line[3].replace(",", ".")
		var p3 := line[5].replace(",", ".")
		if not p1.is_valid_float() or not p2.is_valid_float() or not p3.is_valid_float():
			_on_answers_file_selected("")
			popup_error("(Linha %d) - Peso inválido: %s" % [l, ",".join(line)])
			return null
		
		ini_a.push_back({
			"answer": Lago.parse_answer(line[0]),
			"weight": p1.to_float(),
		})
		ini_b.push_back({
			"answer": Lago.parse_answer(line[2]),
			"weight": p2.to_float(),
		})
		prog.push_back({
			"answer": Lago.parse_answer(line[4]),
			"weight": p3.to_float(),
		})

	return AnswerTable.create(ini_a, ini_b, prog)


func get_participants_db(csv_path: String) -> Dictionary[int, Participante]:
	if csv_path.is_empty():
		return {}

	var csv_file := FileAccess.open(csv_path, FileAccess.READ)
	var l := 0
	var participantes: Dictionary[int, Participante] = {}
	while csv_file.get_position() < csv_file.get_length():
		l += 1

		var line := csv_file.get_csv_line()
		if len(line) != NUM_ARGS_PARTICIPANTES_CSV:
			_on_db_file_selected("")
			popup_error("(Linha %d) - Linha inválida: %s" % [l, ",".join(line)])
			return {}

		if is_header(line):
			continue

		var inscricao := Lago.parse_inscricao(line[0])
		if inscricao == "":
			_on_db_file_selected("")
			popup_error("(Linha %d) - Número de inscrição inválido: %s" % [l, line[0]])
			return {}

		var modalidade := Lago.parse_modalidade(line[3])
		if modalidade == Lago.Modalidade.NONE:
			_on_db_file_selected("")
			popup_error("(Linha %d) - Modalidade inválida: %s" % [l, line[3]])
			return {}

		participantes[inscricao.to_int()] = Participante.create(inscricao, line[1], line[2], modalidade)

	csv_file.close()
	return participantes


func popup_error(err: String) -> void:
	MouseBlocker.show_dialog(err, "Ok :(", "", MouseBlocker.LagostaIcon.SAD)


func _on_directory_selected(new_directory: String) -> void:
	MouseBlocker.hide()
	if new_directory.is_empty() or DirAccess.dir_exists_absolute(new_directory):
		directory_path = new_directory
		sheets_container.clear_sheets()
		sheets_container.add_sheets(SheetReader.init_folder(directory_path))
	folder_line_edit.tooltip_text = "Pasta contendo os scans dos gabaritos.\n%s" % directory_path
	folder_line_edit.text = directory_path # resets to old path if invalid folder


func _on_db_file_selected(new_file: String) -> void:
	MouseBlocker.hide()
	if new_file.is_empty() or (new_file.ends_with(".csv") and FileAccess.file_exists(new_file)):
		db_path = new_file
	db_line_edit.text = db_path # resets to old path if invalid folder
	db_warning_texture.visible = db_path.is_empty()
	db_line_edit.tooltip_text = "Arquivo .csv contendo database de participantes.\n%s" % db_path
	participant_db = get_participants_db(db_path)


func _on_answers_file_selected(new_file: String) -> void:
	MouseBlocker.hide()
	if new_file.is_empty() or (new_file.ends_with(".csv") and FileAccess.file_exists(new_file)):
		answers_path = new_file
	answers_line_edit.text = answers_path # resets to old path if invalid folder
	answers_warning_texture.visible = answers_path.is_empty()
	answers_line_edit.tooltip_text = "Arquivo .csv contendo o gabarito oficial das provas.\n%s" % answers_path
	answer_table = get_answer_table(answers_path)


func _on_folder_pick_button_pressed() -> void:
	MouseBlocker.show_empty()
	folder_dialog.show()


func _on_db_file_pick_button_pressed() -> void:
	MouseBlocker.show_empty()
	db_file_dialog.show()


func _on_answers_file_pick_button_pressed() -> void:
	MouseBlocker.show_empty()
	answers_file_dialog.show()


func _on_dialog_canceled() -> void:
	MouseBlocker.hide()


func _on_read_button_id_pressed(id: int) -> void:
	MouseBlocker.show_dialog(
		"Tem certeza?\nIsso vai resetar quaisquer alterações nas imagens relidas!",
		"Sim",
		"Cancelar",
		MouseBlocker.LagostaIcon.SURPRISED,
	)
	MouseBlocker.ok_pressed.connect(func(): _on_read_confirm(id), CONNECT_ONE_SHOT)


func _on_read_confirm(id: int) -> void:
	var reading_params := ReadingParams.new() # reset to default reading parameters
	match id:
		0: # ler atual
			if info_panel.tracked_button:
				info_panel.tracked_button.reading_params = reading_params
				info_panel.tracked_button.info = SheetReader.read(
					info_panel.tracked_button.info.file_path,
					ReadingParams.new(),
					participant_db,
					answer_table,
				)
				info_panel.tracked_button.update_display()
				info_panel.update_info()
		1: # ler selecionados
			var selected := sheets_container.get_checked()
			var paths: Array[String] = []
			for button in selected:
				paths.push_back(button.info.file_path)
			read_threaded(paths, selected)
		2: # ler tudo
			var all := sheets_container.get_all()
			var paths: Array[String] = []
			for button in all:
				paths.push_back(button.info.file_path)
			read_threaded(paths, all)


var reading: bool = false
var thread: Thread
func _process(_delta: float) -> void:
	if reading:
		print('cu')
		MouseBlocker.set_progress(SheetReader.counter)


func read_threaded(paths: Array[String], buttons: Array[ParticipantButton]) -> void:
	thread = Thread.new()
	
	var f := func() -> void:
		var readings := SheetReader.read_many(paths, participant_db, answer_table)
		update_readings.call_deferred(readings, buttons)
		MouseBlocker.hide.call_deferred()
		reading = false
	
	MouseBlocker.show_loading("Lendo gabaritos...", true, len(paths))
	reading = true
	thread.start(f)

func update_readings(readings: Array[Reading], buttons: Array[ParticipantButton]) -> void:
	for i in range(len(readings)):
		buttons[i].info = readings[i]
		buttons[i].update_display()
		if buttons[i] == info_panel.tracked_button:
			info_panel.update_info()

func _on_folder_line_edit_editing_toggled(toggled_on: bool) -> void:
	if not toggled_on and folder_line_edit.text != directory_path:
		_on_directory_selected(folder_line_edit.text)


func _on_db_line_edit_editing_toggled(toggled_on: bool) -> void:
	if not toggled_on and db_line_edit.text != db_path:
		_on_db_file_selected(db_line_edit.text)


func _on_answers_line_edit_editing_toggled(toggled_on: bool) -> void:
	if not toggled_on and answers_line_edit.text != answers_path:
		_on_answers_file_selected(answers_line_edit.text)


func _on_sheets_container_participant_clicked(participant: ParticipantButton, toggled_on: bool) -> void:
	if not toggled_on:
		reading_h_split.hide()
		info_panel.track(null)
		sheet_preview_texture.texture = null
	else:
		reading_h_split.show()
		info_panel.track(participant)
		sheet_preview_texture.texture = participant.info.get_processed_texture(participant.reading_params)


func _on_reset_params_button_pressed() -> void:
	gamma_spin_box.value = 3.0
	threshold_spin_box.value = 30
