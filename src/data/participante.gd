class_name Participante


var inscricao: String
var nome: String
var escola: String
var modalidade: Lago.Modalidade


func _init(_inscricao: String, _nome: String, _escola: String, _modalidade: Lago.Modalidade) -> void:
	self.inscricao = _inscricao
	self.nome = _nome
	self.escola = _escola
	self.modalidade = _modalidade


func to_sheet(fase: Lago.Fase, edicao: String, scale: float) -> AnswerSheet:
	return AnswerSheet.create(
		inscricao,
		nome,
		escola,
		modalidade,
		fase,
		edicao,
		scale
	)
