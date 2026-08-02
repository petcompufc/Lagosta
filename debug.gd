extends Control


func _ready() -> void:
	var tex := ImageTexture.create_from_image(
		SheetWriter.create_answer_sheet(
			"12345678",
			"Aluno 1",
			"Escola 1",
			"Iniciação A",
			"1",
			"2026",
		)
	)
	%ImageReader.process_image()
	print(%ImageReader.read_barcode())
	%TRect1.texture = tex
	%TRect2.texture = %ImageReader.create_texture_processed()
