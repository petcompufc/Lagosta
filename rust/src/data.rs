use godot::prelude::*;
use std::fmt::Display;

use crate::answer_sheet::AnswerSheet;

const MAX_CHARS_SCHOOL: usize = 60;
const MAX_CHARS_NAME: usize = 54;

#[derive(GodotConvert, Var, Export, Default, Clone, Debug, Copy)]
#[godot(via=u8)]
#[repr(u8)]
pub enum OCIModalidade {
    #[default]
    IniA = 0,
    IniB = 1,
    Prog = 2,
}

#[derive(GodotConvert, Var, Export, Default, Clone, Debug, Copy)]
#[godot(via=u8)]
#[repr(u8)]
pub enum OCIFase {
    #[default]
    Fase1 = 0,
    Fase2 = 1,
    Fase3 = 2,
}

impl OCIModalidade {
    pub fn char(&self) -> &str {
        match self {
            Self::IniA => "a",
            Self::IniB => "b",
            Self::Prog => "p",
        }
    }
}

impl OCIFase {
    pub fn char(&self) -> &str {
        match self {
            Self::Fase1 => "1",
            Self::Fase2 => "2",
            Self::Fase3 => "3",
        }
    }
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
            Self::Fase3 => write!(f, "3"),
        }
    }
}

#[derive(GodotClass, Clone)]
#[class(init)]
pub struct Participante {
    #[var]
    pub inscricao: GString,
    #[var]
    pub nome: GString,
    #[var]
    pub escola: GString,
    #[var]
    pub modalidade: OCIModalidade,
}

#[godot_api]
impl Participante {
    #[func]
    pub fn create(
        inscricao: GString,
        nome: GString,
        escola: GString,
        modalidade: OCIModalidade,
    ) -> Gd<Self> {
        Gd::from_object(Self {
            inscricao,
            nome: cramp(&nome.to_string(), MAX_CHARS_NAME).to_gstring().to_upper(),
            escola: cramp(&escola.to_string(), MAX_CHARS_SCHOOL).to_gstring().to_upper(),
            modalidade,
        })
    }

    #[func]
    pub fn create_texture(&self, fase: OCIFase, edicao: GString, dpi: f32) -> Gd<AnswerSheet> {
        AnswerSheet::create_texture(Gd::from_object(self.clone()), fase, edicao, dpi)
    }
}

unsafe impl Send for Participante {}

unsafe impl Sync for Participante {}

#[inline]
fn cramp(str: &str, limit: usize) -> String {
    str.chars().take(limit).collect()
}
