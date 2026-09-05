use std::fmt::Display;

use crate::data::OCIFase;
use crate::data::Participante;
use crate::reader::params::ReadingParams;
use crate::reader::sheet_reader::SheetReader;
use crate::tools::imgtools::create_godot_texture;
use godot::classes::ImageTexture;
use godot::prelude::*;

const TOTAL_ITEM_COUNT: usize = 20;
pub type Answers = [Answer; TOTAL_ITEM_COUNT];

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

#[derive(GodotConvert, Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
#[godot(via = u8)]
pub enum ReadingError {
    #[default]
    Todo,
}

impl Display for ReadingError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

/// Expects dict in format: `{answer: Answer, weight: float}
#[derive(GodotClass, Clone, Debug)]
#[class(no_init)]
pub struct AnswerTable {
    #[var]
    pub ini_a: Array<AnyDictionary>,
    #[var]
    pub ini_b: Array<AnyDictionary>,
    #[var]
    pub prog: Array<AnyDictionary>,
}

#[godot_api]
impl AnswerTable {
    #[func]
    fn create(
        ini_a: Array<AnyDictionary>,
        ini_b: Array<AnyDictionary>,
        prog: Array<AnyDictionary>,
    ) -> Gd<Self> {
        Gd::from_object(Self { ini_a, ini_b, prog })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
pub struct Reading {
    #[var]
    participante: Gd<Participante>,
    #[var]
    fase: OCIFase,
    #[var]
    errors: Array<ReadingError>,
    #[var]
    score: f32,

    file_path: String,
    answers: Answers,
}

#[godot_api]
impl Reading {
    pub fn new(
        participante: Participante,
        file_path: GString,
        fase: OCIFase,
        answers: Answers,
        score: f32,
        errors: Array<ReadingError>,
    ) -> Self {
        Self {
            participante: Gd::from_object(participante),
            file_path: file_path.to_string(),
            fase,
            answers,
            score,
            errors,
        }
    }

    #[func]
    pub fn get_file(&self) -> GString {
        self.file_path.to_gstring().get_file()
    }

    #[func]
    pub fn get_answers(&self) -> Array<Answer> {
        Array::from_iter(self.answers)
    }

    #[func]
    pub fn set_answer(&mut self, idx: u8, answer: Answer) {
        self.answers[idx as usize] = answer;
    }

    #[func]
    pub fn set_answers(&mut self, answers: Array<Answer>) {
        for (i, a) in answers.iter_shared().enumerate() {
            self.answers[i] = a
        }
    }

    #[func]
    pub fn get_neg_texture(
        &self,
        reading_parameters: Gd<ReadingParams>,
    ) -> Option<Gd<ImageTexture>> {
        create_godot_texture(&SheetReader::neg_image(
            self.file_path.to_string(),
            reading_parameters.bind().gamma,
        )?)
    }

    #[func]
    pub fn get_processed_texture(
        &self,
        reading_parameters: Gd<ReadingParams>,
    ) -> Option<Gd<ImageTexture>> {
        create_godot_texture(&SheetReader::processed_image(
            self.file_path.to_string(),
            reading_parameters.bind().gamma,
            reading_parameters.bind().threshold,
        )?)
    }
}
