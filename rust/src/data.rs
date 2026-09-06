use godot::prelude::*;
use std::fmt::Display;

use crate::generator::AnswerSheet;

const MAX_CHARS_SCHOOL: usize = 60;
const MAX_CHARS_NAME: usize = 54;

#[derive(
    GodotConvert, Var, Export, Default, Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord,
)]
#[godot(via=u8)]
#[repr(u8)]
pub enum OCIModalidade {
    #[default]
    IniA = 0,
    IniB = 1,
    Prog = 2,
    None = 3,
}

#[derive(
    GodotConvert, Var, Export, Default, Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord,
)]
#[godot(via=u8)]
#[repr(u8)]
pub enum OCIFase {
    #[default]
    Fase1 = 0,
    Fase2 = 1,
    Fase3 = 2,
    None = 3,
}

impl OCIModalidade {
    pub fn from_char(s: char) -> Self {
        match s.to_ascii_lowercase() {
            'a' => Self::IniA,
            'b' => Self::IniB,
            'p' => Self::Prog,
            _ => Self::None,
        }
    }

    pub fn char(&self) -> &str {
        match self {
            Self::IniA => "a",
            Self::IniB => "b",
            Self::Prog => "p",
            Self::None => "-",
        }
    }
}

impl OCIFase {
    pub fn from_char(s: char) -> Self {
        match s {
            '1' => Self::Fase1,
            '2' => Self::Fase2,
            '3' => Self::Fase3,
            _ => Self::None,
        }
    }

    pub fn char(&self) -> &str {
        match self {
            Self::Fase1 => "1",
            Self::Fase2 => "2",
            Self::Fase3 => "3",
            Self::None => "-",
        }
    }
}

impl Display for OCIModalidade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IniA => write!(f, "Iniciação A"),
            Self::IniB => write!(f, "Iniciação B"),
            Self::Prog => write!(f, "Programação"),
            Self::None => write!(f, "-"),
        }
    }
}

impl Display for OCIFase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fase1 => write!(f, "1"),
            Self::Fase2 => write!(f, "2"),
            Self::Fase3 => write!(f, "3"),
            Self::None => write!(f, "-"),
        }
    }
}

#[derive(GodotClass, Clone, Default)]
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
            nome: cramp(&nome.to_string(), MAX_CHARS_NAME)
                .to_gstring()
                .to_upper(),
            escola: cramp(&escola.to_string(), MAX_CHARS_SCHOOL)
                .to_gstring()
                .to_upper(),
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

/// why the fuck did i name this "cramp"?
#[inline]
fn cramp(str: &str, limit: usize) -> String {
    str.chars().take(limit).collect()
}
