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
    let mut fs = FileAccess::open(
        "res://assets/templates/gabarito_automatico.svg",
        ModeFlags::READ,
    )
    .unwrap();
    let s = fs.get_as_text().to_string();
    fs.close();
    s
});

static FONT_DATA: LazyLock<Arc<FontDatabase>> = LazyLock::new(|| {
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

#[derive(GodotClass)]
#[class(init, singleton)]
struct SheetWriter {
    base: Base<Object>,
}

#[godot_api]
impl SheetWriter {
    #[func]
    fn create_answer_sheet(
        id: GString,
        participante: GString,
        school: GString,
        modality: GString,
        phase: GString,
        edition: GString,
    ) -> Option<Gd<GDImage>> {
        let barcode = Self::create_barcode(id.into());
        let png64 = Self::encode_base64(&barcode, ExtendedColorType::La8);
        let svg_string = SVG_TEMPLATE
            .replace("{BARCODE}", png64.as_str())
            .replace("{PARTICIPANTE}", &participante.to_string())
            .replace("{ESCOLA}", &school.to_string())
            .replace("{MODALIDADE}", &modality.to_string())
            .replace("{FASE}", &phase.to_string())
            .replace("{EDICAO}", &edition.to_string());

        let imgdata = Self::decode_svg(&svg_string, 4.0);
        GDImage::create_from_data(
            imgdata.width() as i32,
            imgdata.height() as i32,
            false,
            Format::RGBA8,
            &imgdata.as_ref().into(),
        )
    }

    fn create_barcode(text: String) -> GrayAlphaImage {
        let aztec = zxingcpp::create(zxingcpp::BarcodeFormat::Aztec)
            .from_str(text)
            .ok()
            .unwrap()
            .to_image_with(&zxingcpp::write().scale(5))
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

    fn decode_svg(svg_string: &str, scale: f32) -> RgbaImage {
        let mut svg_options = Options::default();
        svg_options.fontdb = FONT_DATA.clone();
        svg_options.dpi = 96.0 * scale;

        let (width, height) = (
            (BASE_TEMPLATE_WIDTH as f32 * scale).floor() as u32,
            (BASE_TEMPLATE_HEIGHT as f32 * scale).floor() as u32,
        );
        let mut pix = Pixmap::new(width, height).unwrap();

        let tree = Tree::from_str(svg_string, &svg_options).unwrap();
        resvg::render(&tree, Transform::default(), &mut pix.as_mut());

        RgbaImage::from_raw(width, height, pix.take()).expect("eita porra!")
    }

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
