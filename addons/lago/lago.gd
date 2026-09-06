## Classe auxiliar da biblioteca Lago. Contém constantes que devem ser utilizadas
## como [code]enum[/code]s junto das funções da biblioteca.
class_name Lago
extends Object

enum Modalidade {
	INI_A = 0,
	INI_B = 1,
	PROG  = 2,
	NONE  = 3,
}

enum Fase {
	FASE_1 = 0,
	FASE_2 = 1,
	FASE_3 = 2,
	NONE   = 3,
}


static func parse_modalidade(input: String) -> Modalidade:
	match input.to_lower():
		"a", "1": return Modalidade.INI_A
		"b", "2": return Modalidade.INI_B
		"p", "3": return Modalidade.PROG
		_: return Modalidade.NONE


static func parse_inscricao(input: String) -> String:
	if not input.is_valid_int():
		return ""
	return "%08d" % input.to_int()
