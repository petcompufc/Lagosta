use std::sync::LazyLock;

use base64::{Engine as _, engine::general_purpose as b64};
use godot::{
    classes::{FileAccess, Image as GDImage, file_access::ModeFlags},
    prelude::*,
};
use imageproc::image::{self, DynamicImage, GrayAlphaImage, GrayImage, ImageEncoder};

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

#[derive(GodotClass)]
#[class(init, singleton)]
struct BarcodeWriter {
    base: Base<Object>,
}

#[godot_api]
impl BarcodeWriter {
    #[func]
    fn create_answer_sheet(
        id: GString,
        name: GString,
        school: GString,
        modality: GString,
        phase: GString,
        edition: GString,
    ) -> Gd<GDImage> {
        let mut img = GDImage::new_gd();
        let barcode = Self::create_barcode(id.into());

        let png64 = Self::png_base64(&barcode);
        img.load_svg_from_string(
            &SVG_TEMPLATE
                .replace("{BARCODE}", png64.as_str())
                .replace("{NOME}", &name.to_string())
                .replace("{ESCOLA}", &school.to_string())
                .replace("{MODALIDADE}", &modality.to_string())
                .replace("{FASE}", &phase.to_string())
                .replace("{EDICAO}", &edition.to_string())
                .to_gstring(),
        );
        img
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

    fn png_base64(img: &GrayAlphaImage) -> String {
        let mut pngvec = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut pngvec);
        encoder
            .write_image(
                img,
                img.width(),
                img.height(),
                image::ExtendedColorType::La8,
            )
            .unwrap();
        b64::STANDARD.encode(pngvec)
    }
}
