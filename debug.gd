extends Control

func _ready() -> void:
	%TRect1.texture = load("res://assets/lagosta.png")
	
	var img := ImageReader.load_image(ProjectSettings.globalize_path("res://assets/lagosta.png"))
	%TRect2.texture = ImageTexture.create_from_image(img)
