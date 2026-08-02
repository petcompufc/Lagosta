use godot::prelude::*;

#[derive(GodotConvert, Var, Export, Default, Clone, Debug)]
#[godot(via=GString)]
pub enum Modalidade {
    #[default]
    IniA,
    IniB,
    Prog,
}

#[derive(GodotConvert, Var, Export, Default, Clone, Debug)]
#[godot(via=GString)]
pub enum Fase {
    #[default]
    Fase1,
    Fase2,
}

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
    modalidade: Modalidade,
    #[var]
    fase: Fase,
}

#[godot_api]
impl Reading {
    #[func]
    fn create(
        nome: GString,
        escola: GString,
        cpf: GString,
        modalidade: Modalidade,
        fase: Fase,
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
