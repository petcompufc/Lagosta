@tool
extends Control

@onready var svc: SubViewportContainer = %SubViewportContainer

func _on_resized() -> void:
	svc.scale.x = size.x / svc.size.x
	svc.scale.y = size.y / svc.size.y
