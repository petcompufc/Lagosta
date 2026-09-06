class_name BallsRect
extends Control

var rect: Rect:
	set(r):
		rect = r
		queue_redraw()

var answers: Array[int] = []

const ITEM_SPACING_X := 0.04735
const ITEM_SPACING_Y := 0.042
const ACTIVE_COLOR := Color(Color.GREEN, 0.5)
const INACTIVE_COLOR := Color(Color.RED, 0.5)


const ITEM_GROUPS: Array[Dictionary] = [
	{
		"item01a_x": 0.193147034,
		"item01a_y": 0.566997519,
	},
	{
		"item01a_x": 0.475519632,
		"item01a_y": 0.566997519,
	},
];

func _ready() -> void:
	if answers.is_empty():
		answers.resize(20)
		answers.fill(5)
	queue_redraw()

func _draw() -> void:
	if not rect:
		return
	var arr := rect.array()
	var item: int = 0
	for g in ITEM_GROUPS:
		var x_init: float = g["item01a_x"]
		var y_init: float = g["item01a_y"]
		for i in range(10):
			var y_lerp := y_init + ITEM_SPACING_Y * i
			for j in range(5):
				var x_lerp := x_init + ITEM_SPACING_X * j
				var p1 := arr[0].lerp(arr[1], x_lerp)
				var p2 := arr[2].lerp(arr[3], x_lerp)
				var center := p1.lerp(p2, y_lerp)
				if answers[item] == j:
					draw_circle(center, 10.0, ACTIVE_COLOR)
				else:
					draw_circle(center, 10.0, INACTIVE_COLOR)
			item += 1
