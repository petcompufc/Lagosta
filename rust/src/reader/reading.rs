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

#[derive(GodotClass, Clone, Debug)]
#[class(no_init)]
pub struct AnswerTable {
    pub ini_a: Vec<(Answer, f32)>,
    pub ini_b: Vec<(Answer, f32)>,
    pub prog: Vec<(Answer, f32)>,
}

#[godot_api]
impl AnswerTable {
    #[func]
    /// Expects dict in format: `{answer: Answer, weight: float}
    fn create(
        ini_a: Array<AnyDictionary>,
        ini_b: Array<AnyDictionary>,
        prog: Array<AnyDictionary>,
    ) -> Gd<Self> {
        Gd::from_object(Self {
            ini_a: Self::array_to_table(ini_a),
            ini_b: Self::array_to_table(ini_b),
            prog: Self::array_to_table(prog),
        })
    }

    fn array_to_table(array: Array<AnyDictionary>) -> Vec<(Answer, f32)> {
        array
            .iter_shared()
            .map(|dict| {
                (
                    dict.get("answer")
                        .unwrap_or(Answer::None.to_variant())
                        .to::<Answer>(),
                    dict.get("weight").unwrap_or(1.0.to_variant()).to::<f32>(),
                )
            })
            .collect()
    }
}

#[derive(GodotClass, Default)]
#[class(no_init)]
pub struct Reading {
    #[var]
    pub participante: Gd<Participante>,
    #[var]
    pub fase: OCIFase,
    #[var]
    pub errors: Array<ReadingError>,
    #[var]
    pub score: f32,
    #[var]
    pub pdf_page: u32,
    #[var]
    pub file_path: GString,
    pub answers: Answers,
}

unsafe impl Send for Reading {}
unsafe impl Sync for Reading {}

#[godot_api]
impl Reading {
    pub fn new(
        file_path: GString,
        pdf_page: u32,
        participante: Participante,
        fase: OCIFase,
        answers: Answers,
        score: f32,
        errors: Array<ReadingError>,
    ) -> Self {
        Self {
            participante: Gd::from_object(participante),
            file_path,
            fase,
            answers,
            score,
            pdf_page,
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
        let mut imgdata = SheetReader::load_image(self.file_path.to_string())?;
        SheetReader::neg_image(&mut imgdata, reading_parameters.bind().gamma);
        create_godot_texture(&imgdata)
    }

    #[func]
    pub fn get_processed_texture(
        &self,
        reading_parameters: Gd<ReadingParams>,
    ) -> Option<Gd<ImageTexture>> {
        let mut imgdata = SheetReader::load_image(self.file_path.to_string())?;
        SheetReader::process_image(
            &mut imgdata,
            reading_parameters.bind().gamma,
            reading_parameters.bind().threshold,
        );
        create_godot_texture(&imgdata)
    }
}
