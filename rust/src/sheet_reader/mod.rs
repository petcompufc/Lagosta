mod reading;

use godot::classes::{Image as GDImage, ImageTexture, image::Format as GDImageFormat};
use godot::prelude::*;
use image::{DynamicImage, GenericImageView, GrayImage};

use crate::tools::imgproc::*;

#[derive(GodotClass)]
#[class(init, singleton)]
struct SheetReader {
    base: Base<Object>,
}

#[godot_api]
impl SheetReader {
    #[func]
    fn image_hough(path: GString) -> Option<Gd<ImageTexture>> {
        let imgdata = image::open(path.to_string())
            .ok()
            .map(DynamicImage::into_luma8)?;
        Self::create_godot_texture(&imgdata.hough_analysis(-90.0..90.0, 1.0, 0.5).image())
    }

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

        let mut hough_img = imgdata.view(0, 0, 100, 120).to_image();
        hough_img.normalized_gradient().threshold(1);

        let h1 = hough_img.hough_analysis(80.0..100.0, 1.0, 0.5);
        let h2 = hough_img.hough_analysis(-10.0..10.0, 1.0, 0.5);
        let r1 = h1.max();
        let r2 = h2.max();
        let _point = r1.intersection_point(r2);

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

    // fn create_godot_texture_color(imgdata: &RgbImage) -> Option<Gd<ImageTexture>> {
    //     let godot_image = GDImage::create_from_data(
    //         imgdata.width() as i32,
    //         imgdata.height() as i32,
    //         false,
    //         GDImageFormat::RGB8,
    //         &imgdata.as_ref().into(),
    //     )?;
    //     ImageTexture::create_from_image(&godot_image)
    // }
}
