use godot::prelude::*;
use crate::data::OCIModalidade;
use crate::data::OCIFase;

#[derive(GodotClass)]
#[class(no_init)]
pub struct Reading {
    #[var]
    nome: GString,
    #[var]
    escola: GString,
    #[var]
    cpf: GString,
    #[var]
    modalidade: OCIModalidade,
    #[var]
    fase: OCIFase,
}

#[godot_api]
impl Reading {
    #[func]
    fn create(
        nome: GString,
        escola: GString,
        cpf: GString,
        modalidade: OCIModalidade,
        fase: OCIFase,
    ) -> Gd<Self> {
        Gd::from_object(Self {
            nome,
            escola,
            cpf,
            modalidade,
            fase,
        })
    }
}
