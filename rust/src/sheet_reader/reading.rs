use crate::data::OCIFase;
use crate::data::OCIModalidade;
use godot::prelude::*;

#[derive(GodotConvert, Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
#[godot(via = u8)]
enum SheetAnswer {
    #[default]
    None = 0,
    A = 1,
    B = 2,
    C = 3,
    D = 4,
    E = 5,
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
    answers: [SheetAnswer; 20],
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
            answers: [SheetAnswer::default(); 20],
        })
    }

    #[func]
    fn get_file(&self) -> GString {
        self.file_path.to_gstring().get_file()
    }

    #[func]
    fn get_answers(&self) -> Array<SheetAnswer> {
        Array::from_iter(self.answers)
    }

    #[func]
    fn set_answer(&mut self, idx: u8, answer: SheetAnswer) {
        self.answers[idx as usize] = answer;
    }

    #[func]
    fn set_answers(&mut self, answers: Array<SheetAnswer>) {
        for (i, a) in answers.iter_shared().enumerate() {
            self.answers[i] = a
        }
    }
}
