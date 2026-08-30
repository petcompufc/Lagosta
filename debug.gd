extends Control

@onready var gamma_sb: SpinBox = %GammaSB
@onready var threshold_sb: SpinBox = %ThresholdSB
@onready var option_button: OptionButton = %OptionButton

var dir := "/home/julia/tmp/lagteste/gabaritos"

func _ready() -> void:
	_on_file_dialog_dir_selected(dir)


func update_texture() -> void:
	var file := "%s/%s" % [dir, option_button.get_item_text(option_button.selected)]
	var gamma := gamma_sb.value
	var threshold: int = floori(threshold_sb.value)
	
	var time := Time.get_ticks_usec()
	var tex := SheetReader.process_image(file, gamma, threshold)
	print("%.2fs" % ((Time.get_ticks_usec() - time) / 1e6))
	
	#var time = Time.get_ticks_usec()
	#var tex := SheetReader.image_hough(file)
	#print("Hough: %.2fs" % ((Time.get_ticks_usec() - time) / 1e6))
	
	%TextureRect.texture = tex


func _input(event: InputEvent) -> void:
	if event.is_action("ui_left") and event.is_pressed():
		option_button.select(clampi(option_button.selected - 1, 0, option_button.item_count - 1))
		update_texture()
	elif event.is_action("ui_right") and event.is_pressed():
		option_button.select(clampi(option_button.selected + 1, 0, option_button.item_count - 1))
		update_texture()


func _on_folder_button_pressed() -> void:
	%FileDialog.show()


func _on_file_dialog_dir_selected(new_dir: String) -> void:
	dir = new_dir
	option_button.clear()
	var files := DirAccess.get_files_at(dir)
	files.sort()
	for file in files:
		if file.ends_with(".png") or file.ends_with(".jpg") or file.ends_with(".jpeg"):
			option_button.add_item(file)
	option_button.select(0)
	update_texture()
