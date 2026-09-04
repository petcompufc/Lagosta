use godot::prelude::*;
use image::{DynamicImage, GenericImageView, GrayImage, imageops};
use zxingcpp::BarcodeFormat;

use crate::data::{OCIFase, OCIModalidade, Participante};
use crate::reader::params::{ItemGroup, ReadingParams, Rect};
use crate::reader::reading::{Answer, Reading};
use crate::tools::imgproc::*;
use crate::tools::imgtools::{clear_transparent, fit_image_to};

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

#[derive(GodotClass)]
#[class(init, singleton)]
struct SheetReader {
    base: Base<Object>,
}

#[godot_api]
impl SheetReader {
    fn processed_image(path: String, gamma: f32, threshold: u8) -> Option<GrayImage> {
        let mut imgdata = image::open(path).ok().map(DynamicImage::into_luma_alpha8)?;
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

        Some(imgdata)
    }

    #[func]
    fn read(
        path: GString,
        reading_params: Gd<ReadingParams>,
        participants_db: Dictionary<i32, Gd<Participante>>,
    ) -> Option<Gd<Reading>> {
        // TODO: errors and warnings
        let imgdata = Self::processed_image(path.to_string(), 3.0, 30)?;

        let rect: Rect = match reading_params.bind().rect.clone() {
            Some(r) => *r.bind(),
            None => Self::get_rect(&imgdata),
        };

        // Lê as respostas do gabarito
        let answers: Vec<Answer> = ITEM_GROUPS
            .iter()
            .flat_map(|ig| Self::read_item_group(&imgdata, ig.clone(), rect, 7, 6))
            .collect();

        // Lê o código QR
        let reader = zxingcpp::read().formats([BarcodeFormat::Aztec]);
        let text;
        let (inscricao, nome, escola, modalidade, fase) = if let Ok(barcodes) =
            reader.from(&imgdata)
            && let Some(barcode) = barcodes.first()
        {
            text = barcode.text();
            let modalidade = OCIModalidade::from_char(text.chars().nth(0).unwrap_or('-'));
            let fase = OCIFase::from_char(text.chars().nth(1).unwrap_or('-'));
            let inscricao = if text.len() > 2 {
                &text[2..]
            } else {
                "00000000" // TODO: type inscrição && impl default?
            };

            let i = inscricao.parse::<i32>().unwrap();
            let (nome, escola) = if let Some(participante) = participants_db.get(i) {
                let p = participante.bind();
                (p.nome.clone(), p.escola.clone())
            } else {
                ("".to_gstring(), "".to_gstring())
            };

            (inscricao, nome, escola, modalidade, fase)
        } else {
            (
                "00000000",
                "".to_gstring(),
                "".to_gstring(),
                OCIModalidade::None,
                OCIFase::None,
            )
        };

        Some(Gd::from_object(Reading::new(
            path,
            inscricao.to_gstring(),
            nome,
            escola,
            modalidade,
            fase,
            *answers.as_array().unwrap(),
        )))
    }

    #[must_use]
    fn get_rect(imgdata: &GrayImage) -> Rect {
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

                (point.0, point.1)
            })
            .collect();

        Rect {
            p1: Vector2::new(corners[0].0, corners[0].1),
            p2: Vector2::new(corners[1].0, corners[1].1),
            p3: Vector2::new(corners[2].0, corners[2].1),
            p4: Vector2::new(corners[3].0, corners[3].1),
        }
    }

    #[must_use]
    fn read_circle(image: &GrayImage, x: u32, y: u32, item_radius: u32) -> u32 {
        let (x, y, radius): (i32, i32, i32) = (x as i32, y as i32, item_radius as i32);
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
                if image.as_raw()[read_x as usize + offset_y] == 255 {
                    count += 1
                }
            }
        }

        count
    }

    #[allow(dead_code)]
    #[must_use]
    fn read_item_group(
        image: &GrayImage,
        item_group: ItemGroup,
        rect: Rect,
        item_radius: u32,
        count_threshold: u32,
    ) -> [Answer; ITEM_COUNT as usize] {
        // TODO: Check for multiple marks
        std::array::from_fn(|i| {
            let y_lerp = item_group.item01a_y + item_group.item_spacing_y * i as f32;
            (0..CHOICE_COUNT)
                .filter_map(|c| {
                    let x_lerp = item_group.item01a_x + item_group.item_spacing_x * c as f32;

                    let vx_top = rect.p1.lerp(rect.p2, x_lerp);
                    let vx_bottom = rect.p3.lerp(rect.p4, x_lerp);
                    let item_pos = vx_top.lerp(vx_bottom, y_lerp);

                    let reading =
                        Self::read_circle(image, item_pos.x as u32, item_pos.y as u32, item_radius);

                    if reading > count_threshold {
                        Some((c, reading))
                    } else {
                        None
                    }
                })
                .max_by(|(_, c1), (_, c2)| c1.cmp(c2))
                .map(|(c, _)| Answer::from_u8(c))
                .unwrap_or(Answer::None)
        })
    }
}
