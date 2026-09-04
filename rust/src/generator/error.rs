use std::fmt::Display;

use image::ImageError;
use krilla::error::KrillaError;
use resvg::usvg::Error as USVGError;
use std::io::Error as IoError;
use zxingcpp::Error as ZXingError;

#[derive(Debug)]
#[repr(u8)]
#[allow(dead_code)]
pub enum AnswerSheetError {
    BarcodeCreate(ZXingError),
    BarcodeEncode(ImageError),
    SVGParse(USVGError),
    PDFCreate(KrillaError),
    IoError(IoError),
}

impl From<ZXingError> for AnswerSheetError {
    fn from(value: ZXingError) -> Self {
        Self::BarcodeCreate(value)
    }
}

impl From<ImageError> for AnswerSheetError {
    fn from(value: ImageError) -> Self {
        Self::BarcodeEncode(value)
    }
}

impl From<USVGError> for AnswerSheetError {
    fn from(value: USVGError) -> Self {
        Self::SVGParse(value)
    }
}

impl From<KrillaError> for AnswerSheetError {
    fn from(value: KrillaError) -> Self {
        Self::PDFCreate(value)
    }
}

impl From<IoError> for AnswerSheetError {
    fn from(value: IoError) -> Self {
        Self::IoError(value)
    }
}

impl Display for AnswerSheetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BarcodeCreate(e) => write!(f, "Erro criando código de barras: {e:?}"),
            Self::BarcodeEncode(e) => {
                write!(f, "Erro codificando código de barras em Base64: {e:?}")
            }
            Self::SVGParse(e) => write!(f, "Erro decodificando o SVG gerado: {e:?}"),
            Self::PDFCreate(e) => write!(f, "Erro criando o PDF: {e:?}"),
            Self::IoError(e) => write!(f, "Erro escrevendo pro disco: {e:?}"),
        }
    }
}
