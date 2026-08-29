mod reading;

use godot::classes::{Image as GDImage, ImageTexture, image::Format as GDImageFormat};
use godot::prelude::*;
use image::{DynamicImage, GrayImage};

use crate::tools::imgproc::*;

#[derive(GodotClass)]
#[class(init, singleton)]
struct SheetReader {
    base: Base<Object>,
}

#[godot_api]
impl SheetReader {
    #[func]
    fn process_image(path: GString, gamma: f32, threshold: u8) -> Option<Gd<ImageTexture>> {
        let mut imgdata = image::open(path.to_string())
            .ok()
            .map(DynamicImage::into_luma8)?;

        imgdata
            .neg()
            .gamma(gamma)
            .threshold(threshold)
            .erode(1)
            .dilate(1);

        Self::create_godot_texture(&imgdata)
    }

    #[func]
    fn load_image(path: GString) -> Option<Gd<ImageTexture>> {
        let imgdata = image::open(path.to_string())
            .ok()
            .map(DynamicImage::into_luma8)?;
        Self::create_godot_texture(&imgdata)
    }

    #[allow(dead_code)]
    fn create_godot_texture(imgdata: &GrayImage) -> Option<Gd<ImageTexture>> {
        let godot_image = GDImage::create_from_data(
            imgdata.width() as i32,
            imgdata.height() as i32,
            false,
            GDImageFormat::L8,
            &imgdata.as_ref().into(),
        )?;
        ImageTexture::create_from_image(&godot_image)
    }
}
