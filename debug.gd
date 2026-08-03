extends Control


func _ready() -> void:
	var tex := AnswerSheet.create(
		"12345678",
		"Aluno 1",
		"Escola 1",
		Lago.Modalidade.PROG,
		Lago.Fase.FASE_2,
		"2026",
		4.0
	).create_texture()
	
	var tex2 := AnswerSheet.create(
		"00000000",
		"Júlia Andrade Ramos",
		"UFC",
		Lago.Modalidade.INI_A,
		Lago.Fase.FASE_1,
		"2026",
		4.0
	).create_texture()
	
	%TRect1.texture = tex
	%TRect2.texture = tex2
