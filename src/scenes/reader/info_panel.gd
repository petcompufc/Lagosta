@tool
class_name InfoPanel
extends Panel

const ERROR_LABEL := preload("res://src/scenes/generator/error_label.tscn")

@onready var name_input: LineEdit = %NameInput
@onready var school_input: LineEdit = %SchoolInput
@onready var phase_input: OptionButton = %PhaseInput
@onready var modality_input: OptionButton = %ModalityInput
@onready var items_container: HFlowContainer = %ItemsContainer
@onready var scroll_container: ScrollContainer = %ScrollContainer
@onready var warnings_v_box: VBoxContainer = %WarningsVBox
@onready var outer_warnings_v_box: VBoxContainer = %OuterWarningsVBox

var tracked_button: ParticipantButton = null

func _ready() -> void:
	scroll_container.hide()
	update_warnings_visibility()
	var children := items_container.get_children()
	for i in range(20):
		var item: ItemButton = children[i]
		item.item_selected.connect(_on_item_selected.bind(i))


func track(participant_button: ParticipantButton) -> void:
	tracked_button = participant_button
	if tracked_button:
		scroll_container.show()
		update_info()
	else:
		scroll_container.hide()
		clear_warnings()


func clear_warnings() -> void:
	for child in warnings_v_box.get_children():
		child.queue_free()
	update_warnings_visibility()


func update_warnings_visibility() -> void:
	outer_warnings_v_box.visible = warnings_v_box.get_child_count() > 0


func update_info() -> void:
	if not tracked_button:
		scroll_container.hide()
		return
	
	name_input.text = tracked_button.info.participante.nome
	school_input.text = tracked_button.info.participante.escola
	modality_input.select(modality_input.get_item_index(tracked_button.info.participante.modalidade))
	phase_input.select(phase_input.get_item_index(tracked_button.info.fase))
	
	var children := items_container.get_children()
	for i in range(20):
		var item: ItemButton = children[i]
		var answers := tracked_button.info.get_answers()
		if len(answers) > i:
			item.selected_answer = answers[i]
		else:
			item.selected_answer = 0
	
	clear_warnings()
	for i in range(len(tracked_button.info.errors)):
		var error := tracked_button.info.errors[i]
		var new_error: ErrorLabel = ERROR_LABEL.instantiate()
		new_error.text = error
		new_error.button_pressed.connect(_on_error_dismissed.bind(i))
		warnings_v_box.add_child(new_error)
	update_warnings_visibility()


func _on_item_selected(answer: Lago.Answer, item: int) -> void:
	if tracked_button:
		tracked_button.info.set_answer(item, answer as int)


func _on_name_input_text_changed(new_text: String) -> void:
	if tracked_button:
		tracked_button.info.participante.nome = new_text
		tracked_button.update_display()


func _on_school_input_text_changed(new_text: String) -> void:
	if tracked_button:
		tracked_button.info.participante.escola = new_text


func _on_modality_input_item_selected(index: int) -> void:
	tracked_button.info.participante.modalidade = modality_input.get_item_id(index)


func _on_phase_input_item_selected(index: int) -> void:
	tracked_button.info.fase = phase_input.get_item_id(index)


func _on_id_input_value_changed(value: float) -> void:
	if tracked_button:
		tracked_button.info.participante.inscricao = "%08d" % value
		tracked_button.update_display()


func _on_error_dismissed(error_idx: int) -> void:
	if tracked_button:
		tracked_button.info.errors.remove_at(error_idx)
		tracked_button.update_display()
		update_warnings_visibility()
