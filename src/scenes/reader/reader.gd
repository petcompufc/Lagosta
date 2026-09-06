extends Panel

var directory_path: String = ""
var db_path: String = ""
var answers_path: String = ""

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


func _on_directory_selected(new_directory: String) -> void:
	MouseBlocker.hide()
	if new_directory.is_empty() or DirAccess.dir_exists_absolute(new_directory):
		directory_path = new_directory
	folder_line_edit.tooltip_text = "Pasta contendo os scans dos gabaritos.\n%s" % directory_path
	folder_line_edit.text = directory_path # resets to old path if invalid folder


func _on_db_file_selected(new_file: String) -> void:
	MouseBlocker.hide()
	if new_file.is_empty() or (new_file.ends_with(".csv") and FileAccess.file_exists(new_file)):
		db_path = new_file
	db_line_edit.text = db_path # resets to old path if invalid folder
	db_warning_texture.visible = db_path.is_empty()
	db_line_edit.tooltip_text = "Arquivo .csv contendo database de participantes.\n%s" % db_path


func _on_answers_file_selected(new_file: String) -> void:
	MouseBlocker.hide()
	if new_file.is_empty() or (new_file.ends_with(".csv") and FileAccess.file_exists(new_file)):
		answers_path = new_file
	answers_line_edit.text = answers_path # resets to old path if invalid folder
	answers_warning_texture.visible = answers_path.is_empty()
	answers_line_edit.tooltip_text = "Arquivo .csv contendo o gabarito oficial das provas.\n%s" % answers_path


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


func _on_read_button_id_pressed(_id: int) -> void:
	MouseBlocker.show_dialog(
		"Tem certeza?\nIsso vai resetar quaisquer alterações nas imagens relidas!",
		"Sim",
		"Cancelar",
		MouseBlocker.LagostaIcon.SURPRISED,
	)
	MouseBlocker.ok_pressed.connect(func(): pass, CONNECT_ONE_SHOT)


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
		return
	info_panel.set_info(participant)
	reading_h_split.show()
