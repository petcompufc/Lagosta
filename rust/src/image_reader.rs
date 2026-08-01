use godot::{
    classes::{Image as GodotImage, image::Format},
    prelude::*,
};
use imageproc::image::{self, DynamicImage, RgbImage};

#[derive(GodotClass)]
#[class(base=Node, init)]
struct ImageReader {}

#[godot_api]
impl ImageReader {
    #[func]
    fn load_image(img_path: String) -> Option<Gd<GodotImage>> {
        let img = image::open(img_path).ok()?.into_luma8();
        GodotImage::create_from_data(
            img.width() as i32,
            img.height() as i32,
            false,
            Format::L8,
            &img.into_raw().into(),
        )
    }
}
