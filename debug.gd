extends Control

func _ready() -> void:
	%ImageReader.process_image()
	%TRect1.texture = %ImageReader.create_texture_original()
	%TRect2.texture = %ImageReader.create_texture_processed()
