extends CanvasLayer

signal ok_pressed
signal cancel_pressed

enum LagostaIcon {
	PENCIL,
	SAD,
	SCARED,
	ANGRY,
	SURPRISED,
}

var icons: Dictionary[int, Texture2D] = {
	LagostaIcon.PENCIL: preload("res://assets/icons/lagosta/lagosta_lapis.png"),
	LagostaIcon.SAD: preload("res://assets/icons/lagosta/lagosta_triste.png"),
	LagostaIcon.SCARED: preload("res://assets/icons/lagosta/lagosta_assustada.png"),
	LagostaIcon.ANGRY: preload("res://assets/icons/lagosta/lagosta_raiva.png"),
	LagostaIcon.SURPRISED: preload("res://assets/icons/lagosta/lagosta_surpresa.png"),
}

@onready var loading_image: TextureRect = %LoadingImage
@onready var dialog_image: TextureRect = %DialogImage
@onready var loading_panel: Panel = %LoadingPanel
@onready var dialog_panel: Panel = %DialogPanel
@onready var loading_label: Label = %LoadingLabel
@onready var dialog_label: Label = %DialogLabel
@onready var ok_button: Button = %OkButton
@onready var cancel_button: Button = %CancelButton


func _ready() -> void:
	hide()


func _process(delta: float) -> void:
	loading_image.offset_transform_rotation += 2.0 * delta


func show_dialog(message: String, ok_string: String = "Ok", cancel_string: String = "", icon: LagostaIcon = LagostaIcon.PENCIL) -> void:
	reset_signals()
	dialog_panel.show()
	loading_panel.hide()
	
	dialog_image.texture = icons[icon]
	dialog_label.text = message
	
	ok_button.text = ok_string
	cancel_button.text = cancel_string
	ok_button.visible = not ok_string.is_empty()
	cancel_button.visible = not cancel_string.is_empty()
	
	show()


func show_empty() -> void:
	dialog_panel.hide()
	loading_panel.hide()
	show()


func show_loading(message: String = "Carregando...") -> void:
	reset_signals()
	dialog_panel.hide()
	loading_panel.show()
	
	loading_label.text = message
	
	show()


func reset_signals() -> void:
	for connection in ok_pressed.get_connections():
		ok_pressed.disconnect(connection["callable"])
	for connection in cancel_pressed.get_connections():
		cancel_pressed.disconnect(connection["callable"])


func _on_ok_button_pressed() -> void:
	hide()
	ok_pressed.emit()
	reset_signals()


func _on_cancel_button_pressed() -> void:
	hide()
	cancel_pressed.emit()
	reset_signals()


func _unhandled_input(event: InputEvent) -> void:
	if not visible:
		return
	if dialog_panel.visible and event is InputEventKey and event.is_pressed() and event.keycode == KEY_ESCAPE:
		if cancel_button.visible:
			_on_cancel_button_pressed()
		else:
			_on_ok_button_pressed()
