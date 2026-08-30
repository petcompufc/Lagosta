mod reading;

use godot::classes::{Image as GDImage, ImageTexture, image::Format as GDImageFormat};
use godot::prelude::*;
use image::{DynamicImage, GenericImageView, GrayImage, imageops};
use rayon::iter::{IntoParallelIterator, ParallelExtend, ParallelIterator};

use crate::tools::imgproc::*;

#[derive(GodotClass)]
#[class(init, singleton)]
struct SheetReader {
    base: Base<Object>,
}

const SHEET_WIDTH: u32 = 1264;
const SHEET_HEIGHT: u32 = 900;

const CORNER_SIZE: u32 = 140;
const CORNERS: [(u32, u32); 4] = [
    (0, 0),
    (SHEET_WIDTH - CORNER_SIZE, 0),
    // -100 instead of -CORNER_SIZE for y allows the corner scan to reach further down
    (0, SHEET_HEIGHT - 100),
    (SHEET_WIDTH - CORNER_SIZE, SHEET_HEIGHT - 100),
];

const EXPECTED_HOUGH_COUNT: u32 = 24;

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
        godot_print!("Pre: {}x{}", imgdata.width(), imgdata.height());
        imgdata = fit_image_to(&imgdata, SHEET_WIDTH, CORNERS[3].1 + CORNER_SIZE);
        imgdata
            .neg()
            .gamma(gamma)
            .threshold(threshold)
            .erode(1)
            .dilate(1);

        for corner in CORNERS {
            let mut hough_img = imgdata
                .view(corner.0, corner.1, CORNER_SIZE, CORNER_SIZE)
                .to_image();
            hough_img.normalized_gradient().threshold(1);

            let h1 = hough_img.hough_analysis(80.0..100.0, 1.0, 0.5);
            let h2 = hough_img.hough_analysis(-10.0..10.0, 1.0, 0.5);
            let r1 = h1.closest_to(EXPECTED_HOUGH_COUNT);
            let r2 = h2.closest_to(EXPECTED_HOUGH_COUNT);

            godot_print!("r1: {}, r2: {}", r1.value, r2.value);

            let point = r1.intersection_point(r2);
            draw_sqr(
                &mut imgdata,
                corner.0 + point.0 as u32,
                corner.1 + point.1 as u32,
                5,
            );
        }
        godot_print!("Post: {}x{}", imgdata.width(), imgdata.height());

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

#[must_use]
fn fit_image_to(image: &GrayImage, target_width: u32, target_height: u32) -> GrayImage {
    let scale = target_width as f32 / image.width() as f32;
    let height = (image.height() as f32 * scale).ceil() as u32;
    let resized = imageops::resize(image, target_width, height, imageops::FilterType::Nearest);

    if height > target_height {
        imageops::crop_imm(&resized, 0, 0, target_width, target_height).to_image()
    } else if height < target_height {
        let remainder = (target_height - height) * target_width;
        let mut result_vec = resized.into_vec();
        result_vec.par_extend((0..remainder).into_par_iter().map(|_| 255));
        GrayImage::from_vec(target_width, target_height, result_vec).unwrap()
    } else {
        resized
    }
}

fn draw_sqr(image: &mut GrayImage, x: u32, y: u32, radius: i32) {
    let width = image.width();
    for i in -radius..radius {
        for j in -radius..radius {
            let dy = (y as i32 + j).clamp(0, image.height() as i32 - 1) as usize;
            let dx = (x as i32 + i).clamp(0, image.width() as i32 - 1) as usize;
            image.as_mut()[dy * width as usize + dx] = 128
        }
    }
}
