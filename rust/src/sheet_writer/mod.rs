pub mod answer_sheet;

use crate::{
    data::{OCIFase, OCIModalidade},
    sheet_writer::answer_sheet::AnswerSheet,
    time,
};
use base64::{Engine as _, engine::general_purpose as b64};
use godot::{
    classes::{FileAccess, file_access::ModeFlags},
    prelude::*,
};
use imageproc::image::{self, ExtendedColorType, GrayAlphaImage, ImageEncoder, RgbaImage};
use resvg::{
    tiny_skia::Pixmap,
    usvg::{Options, Transform, Tree, fontdb::Database as FontDatabase},
};
use std::sync::{Arc, LazyLock};

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

/// Criador de gabaritos. Lida com a criação de gabaritos customizados.
#[derive(GodotClass)]
#[class(init, singleton)]
struct SheetWriter {}

#[godot_api]
impl SheetWriter {
    /// Cria a imagem de um gabarito para um aluno com base nos dados fornecidos. \
    /// **Note:** Use as constantes da classe auxiliar [member Lago.*] nos parâmetros
    /// de `modalidade` e `fase`. [u]**Não use strings.**[/u]
    // TODO: error handling - retornar uma data class com possíveis erros ao invés de só a imagem
    #[func]
    fn create_answer_sheet(
        inscricao: GString,
        participante: GString,
        escola: GString,
        modalidade: OCIModalidade,
        fase: OCIFase,
        edicao: GString,
        image_scale: f32,
    ) -> Gd<AnswerSheet> {
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
        let barcode = time!(
            "Barcode",
            Self::into_base64(Self::create_barcode(barcode_str))
        );

        // Insere as informações numa cópia do template SVG
        let svg_string = SVG_TEMPLATE
            .replace("{BARCODE}", barcode.as_str())
            .replace("{PARTICIPANTE}", &participante.to_string())
            .replace("{ESCOLA}", &escola.to_string())
            .replace("{MODALIDADE}", &modalidade.to_string())
            .replace("{FASE}", &fase.to_string())
            .replace("{EDICAO}", &edicao.to_string())
            .replace("{INSCRICAO}", &inscricao.to_string());

        // Decodifica o SVG em um buffer RGBA8 e converte pra uma Image da Godot.
        AnswerSheet::new(Some(Self::decode_svg(&svg_string, image_scale)), array![])
    }

    /// Encoda uma string em um buffer LumaA8 (grayscale + alpha)
    fn create_barcode(text: String) -> GrayAlphaImage {
        let aztec = zxingcpp::create(zxingcpp::BarcodeFormat::Aztec)
            .from_str(text)
            .ok()
            .unwrap()
            .to_image_with(&zxingcpp::write().scale(5)) // TODO: adjust scale
            .ok()
            .unwrap();
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

        GrayAlphaImage::from_raw(width, height, imgdata_u8).unwrap()
    }

    /// Decoda uma string svg em uma buffer RGBA. \
    /// Usa RGBA pq é o padrão do `Pixmap` que o `resvg` usa.
    fn decode_svg(svg_string: &str, scale: f32) -> RgbaImage {
        let mut svg_options = Options::default();
        svg_options.fontdb = FONT_DB.clone();
        svg_options.dpi = 96.0 * scale;
        let (width, height) = (
            (BASE_TEMPLATE_WIDTH as f32 * scale).floor() as u32,
            (BASE_TEMPLATE_HEIGHT as f32 * scale).floor() as u32,
        );

        let mut pix = Pixmap::new(width, height).unwrap();

        // WARN: erros podem acontecer se no parse do template der algum pau no SVG
        let tree = Tree::from_str(svg_string, &svg_options).unwrap();
        resvg::render(&tree, Transform::default(), &mut pix.as_mut());

        RgbaImage::from_raw(width, height, pix.take()).unwrap()
    }

    /// Encoda um buffer de pixels LumaA8 em um PNG Base64
    fn into_base64(img: GrayAlphaImage) -> String {
        let mut pngvec = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut pngvec);
        encoder
            .write_image(
                img.as_ref(),
                img.width(),
                img.height(),
                ExtendedColorType::La8,
            )
            .unwrap();
        b64::STANDARD.encode(pngvec)
    }
}
