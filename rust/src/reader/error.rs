use std::fmt::Display;

use image::ImageError;

#[derive(Debug)]
#[repr(u8)]
#[allow(dead_code)]
pub enum ReaderError {
    BarcodeRead(String),
    ImageError(ImageError),
    DatabaseError(String),
    AlignmentError(String),
    AnswersError(String),
}

impl From<ImageError> for ReaderError {
    fn from(value: ImageError) -> Self {
        Self::ImageError(value)
    }
}

impl Display for ReaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BarcodeRead(e) => write!(f, "Erro lendo código de barras: {e}"),
            Self::ImageError(e) => write!(f, "Erro lendo imagem: {e:?}"),
            Self::DatabaseError(e) => write!(f, "Participante não encontrado na database: {e}"),
            Self::AlignmentError(e) => write!(f, "Possível erro na detecção de alinhamento: {e}"),
            Self::AnswersError(e) => write!(f, "Possível erro na leitura do gabarito: {e}"),
        }
    }
}
