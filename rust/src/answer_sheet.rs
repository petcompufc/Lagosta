use crate::data::{OCIFase, OCIModalidade, Participante};
use base64::{Engine as _, engine::general_purpose as b64};
use godot::{
    classes::{FileAccess, Image as GDImage, ImageTexture, file_access::ModeFlags, image::Format},
    prelude::*,
};
use imageproc::image::{self, ExtendedColorType, GrayAlphaImage, ImageEncoder, RgbaImage};
use resvg::{
    tiny_skia::Pixmap,
    usvg::{Options, Transform, Tree, fontdb::Database as FontDatabase},
};
use std::{
    fmt::Display,
    path::Path,
    sync::{
        Arc, LazyLock,
    },
};

use rayon::prelude::*;

const BASE_TEMPLATE_WIDTH: u32 = 793;
const BASE_TEMPLATE_HEIGHT: u32 = 559;

static SVG_TEMPLATE: LazyLock<String> = LazyLock::new(|| {
    // NOTE: o modo de import do arquivo precisa estar como "Keep File" na Godot
    // pra não dar problema no export final do Lagosta.
    let mut fs = FileAccess::open(
        "res://assets/templates/gabarito_automatico.svg",
        ModeFlags::READ,
    )
    .unwrap();
    let s = fs.get_as_text().to_string();
    fs.close();
    s
});

static FONT_DB: LazyLock<Arc<FontDatabase>> = LazyLock::new(|| {
    // NOTE: o modo de import do arquivo precisa estar como "Keep File" na Godot
    // pra não dar problema no export final do Lagosta.
    let mut file = FileAccess::open(
        "res://assets/templates/fonts/JetBrainsMono[wght].ttf",
        ModeFlags::READ,
    )
    .unwrap();
    let file_length = file.get_length() as i64;
    let buffer = file.get_buffer(file_length).to_vec();
    file.close();

    let mut fontdb = FontDatabase::new();
    fontdb.load_font_data(buffer);
    Arc::new(fontdb)
});

#[derive(GodotConvert, Var, Export, Default, Clone, Debug, Copy)]
#[godot(via=u8)]
#[repr(u8)]
pub enum AnswerSheetError {
    #[default]
    NoError = 0,
    BarcodeCreate = 1,
    BarcodeEncode = 2,
    SVGParse = 3,
}

/// Criador de gabaritos. Lida com a criação de gabaritos customizados.
#[derive(GodotClass)]
#[class(init)]
pub struct AnswerSheet {
    imgdata: Option<RgbaImage>,
    #[var]
    error: AnswerSheetError,
}

#[godot_api]
impl AnswerSheet {
    #[constant]
    const ERROR_NONE: u8 = AnswerSheetError::NoError as u8;
    #[constant]
    const ERROR_BARCODE_CREATE: u8 = AnswerSheetError::BarcodeCreate as u8;
    #[constant]
    const ERROR_BARCODE_ENCODE: u8 = AnswerSheetError::BarcodeEncode as u8;
    #[constant]
    const ERROR_SVG_PARSE: u8 = AnswerSheetError::SVGParse as u8;

    /// Cria a imagem de um gabarito para um aluno com base nos dados fornecidos. \
    /// **Note:** Use as constantes da classe auxiliar [member Lago.*] nos parâmetros
    /// de `modalidade` e `fase`. [u]**Não use strings.**[/u]
    #[func]
    pub fn create(
        inscricao: GString,
        participante: GString,
        escola: GString,
        modalidade: OCIModalidade,
        fase: OCIFase,
        edicao: GString,
        image_scale: f32,
    ) -> Gd<Self> {
        match Self::new_imgdata(
            inscricao,
            participante,
            escola,
            modalidade,
            fase,
            edicao,
            image_scale,
        ) {
            Ok(imgdata) => Gd::from_object(Self {
                imgdata: Some(imgdata),
                error: AnswerSheetError::NoError,
            }),
            Err(error) => Gd::from_object(Self {
                imgdata: None,
                error: error,
            }),
        }
    }

    /// Cria uma textura pra Godot com os dados internos. Dá erro se a imagem não for válida.
    #[func]
    pub fn create_texture(&self) -> Gd<ImageTexture> {
        assert!(
            self.is_valid(),
            "Tried creating texture from invalid answersheet."
        );
        let imgdata = self.imgdata.as_ref().unwrap();
        ImageTexture::create_from_image(
            &GDImage::create_from_data(
                imgdata.width() as i32,
                imgdata.height() as i32,
                false,
                Format::RGBA8,
                &imgdata.as_ref().into(),
            )
            .unwrap(),
        )
        .unwrap()
    }

    /// Checa se a imagem do gabarito é válida - Se ela existe ou se houve um erro na geração.
    #[func]
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.imgdata.is_some()
    }

    #[func]
    #[inline]
    pub fn save_many(
        dir: GString,
        data: Array<Gd<Participante>>,
        fase: OCIFase,
        edicao: GString,
        image_scale: f32,
        _save_png: bool,
        _save_pdf: bool,
        on_done: Callable,
    ) {
        // needed to pass between threads
        let datavec: Vec<Participante> = data.iter_shared().map(|p| p.bind().clone()).collect();
        let dir = dir.to_string();
        let edicao = edicao.to_string();

        let errors: Vec<GString> = datavec
            .into_par_iter()
            .filter_map(|participant| {
                let sheet = participant.to_sheet(fase, "".to_gstring(), image_scale);
                let sheet = sheet.bind();

                if sheet.is_valid() {
                    let imgdata = sheet.imgdata.as_ref().unwrap();
                    let filename = format!(
                        "gab{}_f{}_ed{}.png",
                        participant.inscricao,
                        fase as u8 + 1,
                        edicao
                    );
                    let filepath = Path::new(&dir).join(filename);

                    // Tries creating file
                    let file = match std::fs::File::create(&filepath) {
                        Ok(f) => f,
                        Err(_) => {
                            return Some(format!(
                                "(Insc. {}) - Erro criando arquivo {}",
                                participant.inscricao,
                                filepath.display()
                            ));
                        }
                    };

                    // Tries writing the png to file
                    let encoder = image::codecs::png::PngEncoder::new(file);
                    if encoder
                        .write_image(
                            imgdata,
                            imgdata.width(),
                            imgdata.height(),
                            image::ExtendedColorType::Rgba8,
                        )
                        .is_err()
                    {
                        return Some(format!(
                            "(Insc. {}) - Erro escrevendo arquivo {}",
                            participant.inscricao,
                            filepath.display()
                        ));
                    }
                    None // No error
                } else {
                    // Error generating answer sheet
                    Some(format!(
                        "(Insc. {}) - {}",
                        participant.inscricao,
                        sheet.error.to_string()
                    ))
                }
            })
            .map(|s| s.to_gstring())
            .collect();

        on_done.call_deferred(&[Array::from_iter(errors).to_variant()]);
    }
}

impl AnswerSheet {
    pub fn new_imgdata(
        inscricao: GString,
        participante: GString,
        escola: GString,
        modalidade: OCIModalidade,
        fase: OCIFase,
        edicao: GString,
        image_scale: f32,
    ) -> Result<RgbaImage, AnswerSheetError> {
        // Formato do código de barras: mf00000000
        // m: letra da modalidade (a, b, p)
        // f: número da fase (1, 2, 3)
        let barcode_str = format!(
            "{}{}{}",
            modalidade.char(),
            fase.char(),
            inscricao.to_string()
        );

        // Cria a imagem do código de barras e encoda como um png em base64 pra colocar no svg
        let barcode_img = Self::create_barcode(barcode_str)?;
        let barcode_base64 = Self::into_base64(barcode_img)?;

        // Insere as informações numa cópia do template SVG
        let svg_string = SVG_TEMPLATE
            .replace("{BARCODE}", barcode_base64.as_str())
            .replace("{PARTICIPANTE}", &participante.to_string())
            .replace("{ESCOLA}", &escola.to_string())
            .replace("{MODALIDADE}", &modalidade.to_string())
            .replace("{FASE}", &fase.to_string())
            .replace("{EDICAO}", &edicao.to_string())
            .replace("{INSCRICAO}", &inscricao.to_string());

        // Decodifica o SVG em um buffer RGBA8 e converte pra uma Image da Godot.
        Self::decode_svg(&svg_string, image_scale)
    }

    /// Encoda uma string em um buffer LumaA8 (grayscale + alpha)
    fn create_barcode(text: String) -> Result<GrayAlphaImage, AnswerSheetError> {
        let aztec = zxingcpp::create(zxingcpp::BarcodeFormat::Aztec)
            .from_str(text)
            .map_err(|_| AnswerSheetError::BarcodeCreate)?
            .to_image_with(&zxingcpp::write().scale(5)) // TODO: adjust scale
            .map_err(|_| AnswerSheetError::BarcodeCreate)?;
        let (width, height) = (aztec.width() as u32, aztec.height() as u32);

        // Converts the Luma8 image into LumaA8
        let data_size = (width * height) as usize;
        let imgdata: Vec<u16> = aztec
            .data()
            .into_iter()
            .map(|pixel| {
                // Makes white pixels transparent: pixels will be read as two bytes: alpha|luma.
                // we invert it so black becomes 1|0 and white becomes 0|0.
                // (the bitshift is due to little-endianness)
                !(pixel as u16) << 8
            })
            .collect();

        // Reinterpret the Vec<u16> as Vec<u8> with double the size.
        let imgdata_ptr = imgdata.into_raw_parts().0 as *mut u8;
        let imgdata_u8 = unsafe { Vec::from_raw_parts(imgdata_ptr, data_size * 2, data_size * 2) };

        Ok(GrayAlphaImage::from_raw(width, height, imgdata_u8).unwrap())
    }

    /// Decoda uma string svg em uma buffer RGBA. \
    /// Usa RGBA pq é o padrão do `Pixmap` que o `resvg` usa.
    fn decode_svg(svg_string: &str, scale: f32) -> Result<RgbaImage, AnswerSheetError> {
        let mut svg_options = Options::default();
        svg_options.fontdb = FONT_DB.clone();
        svg_options.dpi = 96.0 * scale;
        let (width, height) = (
            (BASE_TEMPLATE_WIDTH as f32 * scale).floor() as u32,
            (BASE_TEMPLATE_HEIGHT as f32 * scale).floor() as u32,
        );

        let mut pix = Pixmap::new(width, height).unwrap();

        let tree =
            Tree::from_str(svg_string, &svg_options).map_err(|_| AnswerSheetError::SVGParse)?;

        resvg::render(&tree, Transform::default(), &mut pix.as_mut());

        Ok(RgbaImage::from_raw(width, height, pix.take()).unwrap())
    }

    /// Encoda um buffer de pixels LumaA8 em um PNG Base64
    fn into_base64(img: GrayAlphaImage) -> Result<String, AnswerSheetError> {
        let mut pngvec = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut pngvec);
        encoder
            .write_image(
                img.as_ref(),
                img.width(),
                img.height(),
                ExtendedColorType::La8,
            )
            .map_err(|_| AnswerSheetError::BarcodeEncode)?;
        Ok(b64::STANDARD.encode(pngvec))
    }
}

impl Display for AnswerSheetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoError => write!(f, "Nenhum erro."),
            Self::BarcodeCreate => write!(f, "Erro criando código de barras."),
            Self::BarcodeEncode => write!(f, "Erro codificando código de barras para Base64."),
            Self::SVGParse => write!(f, "Erro decodificando o SVG gerado."),
        }
    }
}
