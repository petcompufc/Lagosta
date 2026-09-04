use std::fmt::Display;

use crate::data::OCIFase;
use crate::data::OCIModalidade;
use godot::prelude::*;

#[derive(GodotConvert, Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
#[godot(via = u8)]
pub enum Answer {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    #[default]
    None = 5,
}

impl Display for Answer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::A => write!(f, "a"),
            Self::B => write!(f, "b"),
            Self::C => write!(f, "c"),
            Self::D => write!(f, "d"),
            Self::E => write!(f, "e"),
            Self::None => write!(f, "-"),
        }
    }
}

impl Answer {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::A,
            1 => Self::B,
            2 => Self::C,
            3 => Self::D,
            4 => Self::E,
            _ => Self::None,
        }
    }
}

#[derive(GodotClass)]
#[class(no_init)]
pub struct Reading {
    #[var]
    inscricao: GString,
    #[var]
    nome: GString,
    #[var]
    escola: GString,
    #[var]
    modalidade: OCIModalidade,
    #[var]
    fase: OCIFase,

    file_path: String,
    answers: [Answer; 20],
}

#[godot_api]
impl Reading {
    #[func]
    fn create(
        file_path: GString,
        inscricao: GString,
        nome: GString,
        escola: GString,
        modalidade: OCIModalidade,
        fase: OCIFase,
    ) -> Gd<Self> {
        Gd::from_object(Self {
            inscricao,
            nome,
            escola,
            modalidade,
            fase,
            file_path: file_path.to_string(),
            answers: [Answer::default(); 20],
        })
    }

    #[func]
    fn get_file(&self) -> GString {
        self.file_path.to_gstring().get_file()
    }

    #[func]
    fn get_answers(&self) -> Array<Answer> {
        Array::from_iter(self.answers)
    }

    #[func]
    fn set_answer(&mut self, idx: u8, answer: Answer) {
        self.answers[idx as usize] = answer;
    }

    #[func]
    fn set_answers(&mut self, answers: Array<Answer>) {
        for (i, a) in answers.iter_shared().enumerate() {
            self.answers[i] = a
        }
    }
}
