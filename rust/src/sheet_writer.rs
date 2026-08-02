use crate::data::{OCIFase, OCIModalidade};
use base64::{Engine as _, engine::general_purpose as b64};
use godot::{
    classes::{FileAccess, Image as GDImage, file_access::ModeFlags, image::Format},
    prelude::*,
};
use imageproc::image::{
    self, DynamicImage, ExtendedColorType, GrayAlphaImage, GrayImage, ImageBuffer, ImageEncoder,
    Pixel, RgbaImage,
};
use resvg::{
    tiny_skia::Pixmap,
    usvg::{Options, Transform, Tree, fontdb::Database as FontDatabase},
};
use std::{
    ops::Deref,
    sync::{Arc, LazyLock},
};

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
    /// **Note:** Use as constantes [member Lago.*] na modalidade e fase. [u]**Não use strings.**[/u]
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
    ) -> Option<Gd<GDImage>> {
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
        let barcode = Self::create_barcode(barcode_str);
        let png64 = Self::encode_base64(&barcode, ExtendedColorType::La8);

        // Insere as informações numa cópia do template SVG
        let svg_string = SVG_TEMPLATE
            .replace("{BARCODE}", png64.as_str())
            .replace("{PARTICIPANTE}", &participante.to_string())
            .replace("{ESCOLA}", &escola.to_string())
            .replace("{MODALIDADE}", &modalidade.to_string())
            .replace("{FASE}", &fase.to_string())
            .replace("{EDICAO}", &edicao.to_string())
            .replace("{INSCRICAO}", &inscricao.to_string());

        // Decodifica o SVG em um buffer RGBA8 e converte pra uma Image da Godot.
        let imgdata = Self::decode_svg(&svg_string, image_scale);
        GDImage::create_from_data(
            imgdata.width() as i32,
            imgdata.height() as i32,
            false,
            Format::RGBA8,
            &imgdata.as_ref().into(),
        )
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
        let (width, height) = (aztec.width(), aztec.height());

        let mut imgdata = DynamicImage::ImageLuma8(
            GrayImage::from_vec(width as u32, height as u32, aztec.data()).unwrap(),
        )
        .into_luma_alpha8();

        // (Lum, Alpha) pair
        for pixel in imgdata.chunks_mut(2) {
            // Make white pixels transparent - This relies on overflows:
            // white pixels will have alpha 0 and other pixels will have alpha 0 - 1 = 255
            pixel[1] = ((pixel[0] == 255) as u8).wrapping_sub(1);
        }

        imgdata
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

        // Safety: width & height > 0
        let mut pix = Pixmap::new(width, height).unwrap();

        // WARN: erros podem acontecer se no parse do template der algum pau no SVG
        let tree = Tree::from_str(svg_string, &svg_options).unwrap();
        resvg::render(&tree, Transform::default(), &mut pix.as_mut());

        // Safety: pix has same width & height
        RgbaImage::from_raw(width, height, pix.take()).unwrap()
    }

    /// Encoda um buffer de pixels qualquer em um PNG Base64
    fn encode_base64<P, Container>(
        img: &ImageBuffer<P, Container>,
        color_type: ExtendedColorType,
    ) -> String
    where
        P: Pixel<Subpixel = u8>,
        Container: Deref<Target = [P::Subpixel]>,
    {
        let mut pngvec = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut pngvec);
        encoder
            .write_image(img.as_ref(), img.width(), img.height(), color_type)
            .unwrap();
        b64::STANDARD.encode(pngvec)
    }
}
