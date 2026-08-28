@tool
class_name InfoPanel
extends Panel

@onready var name_input: LineEdit = %NameInput
@onready var school_input: LineEdit = %SchoolInput
@onready var phase_input: OptionButton = %PhaseInput
@onready var modality_input: OptionButton = %ModalityInput
@onready var items_container: HFlowContainer = %ItemsContainer

func set_info(participant_button: ParticipantButton) -> void:
	name_input.text = participant_button.info.nome
	school_input.text = participant_button.info.escola
	modality_input.select(modality_input.get_item_index(participant_button.info.modalidade))
	var children := items_container.get_children()
	for i in range(20):
		var item: ItemButton = children[i]
		if len(participant_button.answers) > i:
			item.selected_answer = participant_button.answers[i]
		else:
			item.selected_answer = 0
	# TODO: fase and SheetReading instead of participant and answers bullshit
