mod reading;

use godot::classes::{Image as GDImage, ImageTexture, image::Format as GDImageFormat};
use godot::prelude::*;
use image::{DynamicImage, GenericImageView, GrayAlphaImage, GrayImage, imageops};
use rayon::iter::{IntoParallelIterator, ParallelExtend, ParallelIterator};
use rayon::slice::{ParallelSliceMut};

use crate::tools::imgproc::*;

#[derive(GodotClass)]
#[class(init, singleton)]
struct SheetReader {
    base: Base<Object>,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
struct ItemGroup {
    item01a_x: f32,
    item01a_y: f32,
    item_spacing_x: f32,
    item_spacing_y: f32,
    num_items: u32,
    num_choices: u32,
}

// A5 proportion
const SHEET_WIDTH: u32 = 1264;
const SHEET_HEIGHT: u32 = 893;

const CORNER_SIZE: u32 = 125;
const CORNER_X2: u32 = SHEET_WIDTH - CORNER_SIZE;
const CORNER_Y2: u32 = SHEET_HEIGHT - CORNER_SIZE;

const CORNERS: [(u32, u32); 4] = [
    (0, 0),
    (CORNER_X2, 0),
    (0, CORNER_Y2),
    (CORNER_X2, CORNER_Y2),
];

const EXPECTED_HOUGH_COUNT: u32 = 24;

#[allow(dead_code)]
#[allow(clippy::excessive_precision)]
// Valores calculados de forma relativa usando uma imagem 1323x932 do gabarito oficial
// como base, levando em conta que a área lida pelo leitor é a área interna demarcada
// pelos marcadores de alinhamento.
const ITEM_GROUPS: [ItemGroup; 2] = [
    // Itens 01 a 10
    ItemGroup {
        item01a_x: 0.193147034,
        item01a_y: 0.563997519,
        item_spacing_x: 0.04735,
        item_spacing_y: 0.042,
        num_items: 10,
        num_choices: 5,
    },
    // Itens 11 a 20
    ItemGroup {
        item01a_x: 0.475519632,
        item01a_y: 0.563997519,
        item_spacing_x: 0.04735,
        item_spacing_y: 0.042,
        num_items: 10,
        num_choices: 5,
    },
];

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
            .map(DynamicImage::into_luma_alpha8)?;
        clear_transparent(&mut imgdata);
        let mut imgdata = imageops::grayscale(&imgdata);

        imgdata = fit_image_to(&imgdata, SHEET_WIDTH, CORNERS[3].1 + CORNER_SIZE);
        // TODO: apply filter only in the reading parts of the image
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

            // TODO: remove larger line blobs from the analysis
            //  - IDEA: dilate(2), remove large blobs, erode(2)
            // TODO: pick lines closest to expected position
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

fn clear_transparent(image: &mut GrayAlphaImage) {
    image.as_mut().par_chunks_mut(2).for_each(|p| {
        if p[1] < 255 {
            p[0] = 255;
            p[1] = 255;
        }
    });
}
