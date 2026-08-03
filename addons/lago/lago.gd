## Classe auxiliar da biblioteca Lago. Contém constantes que devem ser utilizadas
## como [code]enum[/code]s junto das funções da biblioteca.
class_name Lago
extends Object

enum Modalidade {
	INI_A = 0,
	INI_B = 1,
	PROG  = 2,
}

enum Fase {
	FASE_1 = 0,
	FASE_2 = 1,
	FASE_3 = 2,
}


static func parse_modalidade(input: String) -> Modalidade:
	match input.to_lower():
		"a": return Modalidade.INI_A
		"b": return Modalidade.INI_B
		"p": return Modalidade.PROG
		_: return -1
