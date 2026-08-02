extends Control


func _ready() -> void:
	var tex := ImageTexture.create_from_image(
		SheetWriter.create_answer_sheet(
			"12345678",
			"Aluno 1",
			"Escola 1",
			Lago.INI_A,
			Lago.FASE_1,
			"2026",
			4.0
		)
	)
	
	var tex2 := ImageTexture.create_from_image(
		SheetWriter.create_answer_sheet(
			"00000000",
			"Júlia Andrade Ramos",
			"UFC",
			Lago.INI_A,
			Lago.FASE_3,
			"2026",
			4.0
		)
	)
	
	%TRect1.texture = tex
	%TRect2.texture = tex2
