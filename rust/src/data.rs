use std::fmt::Display;

use godot::prelude::*;

#[derive(GodotConvert, Var, Export, Default, Clone, Debug)]
#[godot(via=GString)]
pub enum OCIModalidade {
    #[default]
    IniA,
    IniB,
    Prog,
}

#[derive(GodotConvert, Var, Export, Default, Clone, Debug)]
#[godot(via=GString)]
pub enum OCIFase {
    #[default]
    Fase1,
    Fase2,
}

impl Display for OCIModalidade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IniA => write!(f, "Iniciação A"),
            Self::IniB => write!(f, "Iniciação B"),
            Self::Prog => write!(f, "Programação"),
        }
    }
}

impl Display for OCIFase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fase1 => write!(f, "1"),
            Self::Fase2 => write!(f, "2"),
        }
    }
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
