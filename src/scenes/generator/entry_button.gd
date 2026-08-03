class_name EntryButton
extends Button

var entry_number := 0:
	set(n):
		entry_number = n
		update_text()

var participant: Participante: 
	set(p):
		participant = p
		update_text()


func update_text():
	var nome := ""
	if participant != null:
		nome = participant.nome
	text = "%d · %s" % [entry_number, nome]
