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

enum Answer {
	A = 0,
	B = 1,
	C = 2, 
	D = 3,
	E = 4,
	NONE = 5,
}

const MODALIDADE_CHAR: Array[String] = ["A","B","P","-"]
const FASE_CHAR: Array[String] = ["1","2","3","-"]
const ANSWER_CHAR: Array[String] = ["a","b","c","d","e","-"]


static func parse_modalidade(input: String) -> Modalidade:
	match input.to_lower():
		"a", "1": return Modalidade.INI_A
		"b", "2": return Modalidade.INI_B
		"p", "3": return Modalidade.PROG
		_: return Modalidade.NONE


static func parse_answer(input: String) -> Answer:
	match input.to_lower():
		"a", "1": return Answer.A
		"b", "2": return Answer.B
		"c", "3": return Answer.C
		"d", "4": return Answer.D
		"e", "5": return Answer.E
		_: return Answer.NONE


static func parse_inscricao(input: String) -> String:
	if not input.is_valid_int():
		return ""
	return "%08d" % input.to_int()


static func answers_str(answers: Array[int]) -> String:
	return "".join(answers.map(func(a: int): return ANSWER_CHAR[a]))
