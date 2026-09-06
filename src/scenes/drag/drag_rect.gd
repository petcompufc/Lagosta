class_name DragRect
extends Control

signal points_changed(points: PackedVector2Array)
const DRAG_BUTTON := preload("res://src/scenes/drag/drag_button.tscn")

var buttons: Array[DragButton] = []
@onready var polygon2d: Polygon2D = $Polygon2D

func _ready() -> void:
	for i: int in range(4):
		var col := i % 2
		var row := floori(i/2.0)
		
		var but: DragButton = DRAG_BUTTON.instantiate()
		add_child(but)
		
		var bsize := but.size / 2.0
		but.position.x = size.x * col - bsize.x
		but.position.y = size.y * row - bsize.y
		but.upper_limit_x = size.x
		but.upper_limit_y = size.y
		
		buttons.push_back(but)
		but.button_up.connect(_on_button_released)


func _process(_delta: float) -> void:
	update_points()
	queue_redraw()

func update_points() -> void:
	var bsize := buttons[0].size / 2.0
	polygon2d.polygon = [
		buttons[0].position + bsize,
		buttons[1].position + bsize,
		buttons[3].position + bsize,
		buttons[2].position + bsize,
	]

func set_coords(coords: PackedVector2Array) -> void:
	var bsize := buttons[0].size / 2.0
	for i in range(4):
		buttons[i].position = coords[i] - bsize


func set_global_coords(coords: PackedVector2Array) -> void:
	for i in range(4):
		buttons[i].global_position = coords[i]


func get_points() -> Array[Vector2]:
	return [
		buttons[0].pos(),
		buttons[1].pos(),
		buttons[2].pos(),
		buttons[3].pos(),
	]


func _on_button_released() -> void:
	update_points()
	points_changed.emit(get_points())


func _on_resized() -> void:
	for but in buttons:
		but.upper_limit_x = size.x
		but.upper_limit_y = size.y
		but.clamp_pos()
