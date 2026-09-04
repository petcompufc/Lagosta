mod reading;

use glam::Vec2;
use godot::classes::{Image as GDImage, ImageTexture, image::Format as GDImageFormat};
use godot::prelude::*;
use image::{DynamicImage, GenericImageView, GrayAlphaImage, GrayImage, imageops};
use rayon::prelude::*;
use rayon::slice::ParallelSliceMut;
use zxingcpp::BarcodeFormat;

use crate::sheet_reader::reading::Answer;
use crate::tools::imgproc::*;

// A5 proportion
const SHEET_WIDTH: u32 = 1264;
const SHEET_HEIGHT: u32 = 893;

const CORNER_SIZE: u32 = 125;
const CORNER_X2: u32 = SHEET_WIDTH - CORNER_SIZE;
const CORNER_Y2: u32 = SHEET_HEIGHT - CORNER_SIZE;

const ITEM_COUNT: u8 = 10;
const CHOICE_COUNT: u8 = 5;

const CORNERS: [(u32, u32); 4] = [
    (0, 0),
    (CORNER_X2, 0),
    (0, CORNER_Y2),
    (CORNER_X2, CORNER_Y2),
];

const EXPECTED_HOUGH_COUNT: u32 = 24;

/// Valores calculados de forma relativa usando uma imagem 1323x932 do gabarito oficial
/// como base, levando em conta que a área lida pelo leitor é a área interna demarcada
/// pelos marcadores de alinhamento.
#[allow(dead_code)]
#[allow(clippy::excessive_precision)]
const ITEM_GROUPS: [ItemGroup; 2] = [
    // Itens 01 a 10
    ItemGroup {
        item01a_x: 0.193147034,
        item01a_y: 0.566997519,
        item_spacing_x: 0.04735,
        item_spacing_y: 0.042,
    },
    // Itens 11 a 20
    ItemGroup {
        item01a_x: 0.475519632,
        item01a_y: 0.566997519,
        item_spacing_x: 0.04735,
        item_spacing_y: 0.042,
    },
];

/// Posição relativa de uma tabela de itens no gabarito (em relação aos alinhadores)
#[allow(dead_code)]
#[derive(Clone, Debug)]
struct ItemGroup {
    item01a_x: f32,
    item01a_y: f32,
    item_spacing_x: f32,
    item_spacing_y: f32,
}

#[derive(Clone, Copy)]
struct Rect {
    p1: Vec2,
    p2: Vec2,
    p3: Vec2,
    p4: Vec2,
}

#[derive(GodotClass)]
#[class(init, singleton)]
struct SheetReader {
    base: Base<Object>,
}

#[godot_api]
impl SheetReader {
    #[func]
    fn image_hough_graph(path: GString) -> Option<Gd<ImageTexture>> {
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

        // TODO: apply filter only in the reading parts of the image
        imgdata = fit_image_to(&imgdata, SHEET_WIDTH, CORNERS[3].1 + CORNER_SIZE);
        imgdata
            .neg()
            .gamma(gamma)
            .threshold(threshold)
            .erode(1)
            .dilate(1);

        let corners: Vec<(f32, f32)> = CORNERS
            .iter()
            .map(|corner| {
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

                let point = r1.intersection_point(r2);
                let point = (point.0 + corner.0 as f32, point.1 + corner.1 as f32);

                // draw_sqr(
                //     &mut imgdata,
                //     point.0 as u32,
                //     point.1 as u32,
                //     5,
                // );

                (point.0, point.1)
            })
            .collect();

        let rect = Rect {
            p1: corners[0].into(),
            p2: corners[1].into(),
            p3: corners[2].into(),
            p4: corners[3].into(),
        };

        for (gi, ig) in ITEM_GROUPS.iter().enumerate() {
            let g = read_item_group(&mut imgdata, ig.clone(), rect, 7, 128, 6);
            for (i, item) in g.iter().enumerate() {
                godot_print!("{}. {item}", i + 1 + 10 * gi);
            }
        }

        let reader = zxingcpp::read().formats([BarcodeFormat::Aztec]);
        let barcodes = reader.from(&imgdata).ok()?;
        for barcode in barcodes {
            godot_print!("{}", barcode.text());
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
        result_vec.extend(std::iter::repeat_n(255, remainder as usize).collect::<Vec<u8>>());
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

#[allow(dead_code)]
#[must_use]
fn read_circle(image: &mut GrayImage, x: u32, y: u32, radius: u32, threshold: u8) -> u32 {
    let (x, y, radius): (i32, i32, i32) = (x as i32, y as i32, radius as i32);
    let radiusf = radius as f32;
    let width = image.width() as i32;
    let height = image.height() as i32;

    let mut count = 0;
    for dy in -radius..=radius {
        let read_y = y + dy;
        if read_y < 0 || read_y >= height - 1 {
            continue;
        }
        let offset_y = read_y as usize * width as usize;
        for dx in -radius..=radius {
            // Ignore pixels outside of the circle radius
            if (dx as f32).hypot(dy as f32) > radiusf {
                continue;
            }

            let read_x = x + dx;
            if read_x < 0 || read_x >= width - 1 {
                continue;
            }
            if image.as_raw()[read_x as usize + offset_y] >= threshold {
                count += 1
            }
        }
    }

    count
}

#[allow(dead_code)]
#[must_use]
fn read_item_group(
    image: &mut GrayImage,
    item_group: ItemGroup,
    rect: Rect,
    radius: u32,
    luma_threshold: u8,
    count_threshold: u32,
) -> [Answer; ITEM_COUNT as usize] {
    std::array::from_fn(|i| {
        let y_lerp = item_group.item01a_y + item_group.item_spacing_y * i as f32;
        (0..CHOICE_COUNT)
            .filter_map(|c| {
                let x_lerp = item_group.item01a_x + item_group.item_spacing_x * c as f32;

                let vx_top = rect.p1.lerp(rect.p2, x_lerp);
                let vx_bottom = rect.p3.lerp(rect.p4, x_lerp);
                let item_pos = vx_top.lerp(vx_bottom, y_lerp).as_uvec2();

                let count = read_circle(image, item_pos.x, item_pos.y, radius, luma_threshold);
                if count > count_threshold {
                    // draw_sqr(image, item_pos.x, item_pos.y, 5);
                    Some((c, count))
                } else {
                    None
                }
            })
            .max_by(|(_, c1), (_, c2)| c1.cmp(c2))
            .map(|(c, _)| Answer::from_u8(c))
            .unwrap_or(Answer::None)
    })
}
