extends Control

@onready var gamma_sb: SpinBox = %GammaSB
@onready var threshold_sb: SpinBox = %ThresholdSB
@onready var option_button: OptionButton = %OptionButton

func _ready() -> void:
	for file in DirAccess.get_files_at("./assets/test_scans/"):
		if file.ends_with(".png"):
			option_button.add_item(file)
	option_button.select(12)
	update_texture()


func update_texture() -> void:
	var file := "./assets/test_scans/%s" % option_button.get_item_text(option_button.selected)
	var gamma := gamma_sb.value
	var threshold: int = floori(threshold_sb.value)
	
	print(option_button.get_item_text(option_button.selected))
	var tex := SheetReader.process_image(
		file,
		gamma,
		threshold,
	)
	%TextureRect.texture = tex


func _input(event: InputEvent) -> void:
	if event.is_action("ui_left") and event.is_pressed():
		option_button.select(clampi(option_button.selected - 1, 0, option_button.item_count - 1))
		update_texture()
	elif event.is_action("ui_right") and event.is_pressed():
		option_button.select(clampi(option_button.selected + 1, 0, option_button.item_count - 1))
		update_texture()


func _on_erode_sb_value_changed() -> void:
	pass # Replace with function body.
