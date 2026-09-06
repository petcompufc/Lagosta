use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::Mutex;

use godot::prelude::*;
use image::{DynamicImage, GenericImageView, GrayImage, imageops};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use zxingcpp::BarcodeFormat;

use crate::data::{OCIFase, OCIModalidade, Participante};
use crate::reader::params::{ItemGroup, ReadingParams, Rect};
use crate::reader::reading::{Answer, AnswerTable, Answers, Reading};
use crate::tools::imgtools::{clear_transparent, fit_image_to};
use crate::tools::{dict_to_hashmap, imgproc::*};

// A5 proportion
const SHEET_WIDTH: u32 = 1264;
const SHEET_HEIGHT: u32 = 893;

const CORNER_SIZE: u32 = 125;
const CORNER_X2: u32 = SHEET_WIDTH - CORNER_SIZE;
const CORNER_Y2: u32 = SHEET_HEIGHT - CORNER_SIZE;

const GROUP_ITEM_COUNT: usize = 10;
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
pub struct SheetReader {
    counter: u32,
    base: Base<Object>,
}

#[godot_api]
impl SheetReader {
    #[func]
    pub fn init_folder(path: String) -> Array<Gd<Reading>> {
        let entries = if let Ok(e) = std::fs::read_dir(path) {
            e
        } else {
            return Array::new();
        };

        entries
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                if path.is_file() { Some(path) } else { None }
            })
            .flat_map(Self::init_file)
            .collect()
    }

    pub fn init_file(path: PathBuf) -> Vec<Gd<Reading>> {
        match path.extension() {
            // TODO: check extensions
            Some(_) => {
                vec![Gd::from_object(Reading {
                    file_path: path.to_string_lossy().to_string().to_gstring(),
                    ..Default::default()
                })]
            }
            None => Vec::new(),
        }
    }

    // clear_transparent(imgdata);
    pub fn load_image(path: String) -> Option<GrayImage> {
        let mut imgdata = image::open(path).ok().map(DynamicImage::into_luma_alpha8)?;
        clear_transparent(&mut imgdata);
        Some(imageops::grayscale(&imgdata))
    }

    // TODO: apply filters only in the reading parts of the image
    pub fn neg_image(imgdata: &mut GrayImage, gamma: f32) -> &mut GrayImage {
        *imgdata = imageops::grayscale(imgdata);
        *imgdata = fit_image_to(imgdata, SHEET_WIDTH, SHEET_HEIGHT);
        imgdata.neg().gamma(gamma);
        imgdata
    }

    pub fn process_image(imgdata: &mut GrayImage, gamma: f32, threshold: u8) -> &mut GrayImage {
        Self::neg_image(imgdata, gamma);
        imgdata.threshold(threshold).erode(1).dilate(1);
        imgdata
    }

    #[func]
    pub fn read_many(
        &mut self,
        paths: Array<GString>,
        participants_db: Dictionary<i32, Gd<Participante>>,
        answer_table: Option<Gd<AnswerTable>>,
    ) -> Array<Gd<Reading>> {
        let reading_params = ReadingParams::default();
        let participants_db = dict_to_hashmap(participants_db);
        let answer_table = answer_table.map(|a| a.bind().clone());

        // Reset progress counter
        self.counter = 0;
        let counter = Mutex::new(&mut self.counter);

        paths
            .iter_shared()
            .collect::<Vec<GString>>()
            .into_par_iter()
            .map(|path| {
                let reading = Self::read_internal(
                    path,
                    &reading_params,
                    &participants_db,
                    answer_table.as_ref(),
                )
                .unwrap_or(Reading::default());
                // Increment self counter
                **counter.lock().unwrap() += 1;
                reading
            })
            .collect::<Vec<Reading>>()
            .into_iter()
            .map(Gd::from_object) // UGLY but I DON'T FUCKING CARE
            .collect()
    }

    #[func]
    fn read(
        path: GString,
        reading_params: Gd<ReadingParams>,
        participants_db: Dictionary<i32, Gd<Participante>>,
        answer_table: Option<Gd<AnswerTable>>,
    ) -> Option<Gd<Reading>> {
        let participants_db = dict_to_hashmap(participants_db);
        let answer_table = answer_table.map(|t| t.bind().clone());
        Self::read_internal(
            path,
            reading_params.bind().deref(),
            &participants_db,
            answer_table.as_ref(),
        )
        .map(Gd::from_object)
    }

    fn read_internal(
        path: GString,
        reading_params: &ReadingParams,
        participants_db: &HashMap<i32, Participante>,
        answer_table: Option<&AnswerTable>,
    ) -> Option<Reading> {
        let mut imgdata = Self::load_image(path.to_string())?;

        // Lê o código QR na imagem >original<
        let (participante, fase) = Self::read_barcode(&imgdata, participants_db);

        Self::process_image(&mut imgdata, 3.0, 30);

        // Lê as respostas do gabarito
        let answers = Self::read_answers(&imgdata, reading_params.rect.clone().map(|r| *r.bind()));

        // Calcula pontuação
        let score = if let Some(at) = answer_table {
            Self::get_score(&answers, at.clone(), participante.modalidade)
        } else {
            0.0
        };

        // TODO: errors and warnings
        Some(Reading::new(
            path,
            0,
            participante,
            fase,
            *answers.as_array().unwrap(),
            score,
            Array::new(),
        ))
    }

    #[must_use]
    fn read_answers(imgdata: &GrayImage, reading_rect: Option<Rect>) -> Answers {
        let rect: Rect = match reading_rect {
            Some(r) => r,
            None => Self::get_rect(imgdata),
        };

        // Lê as respostas do gabarito
        *ITEM_GROUPS
            .iter()
            .flat_map(|ig| Self::read_item_group(imgdata, ig.clone(), &rect, 7, 6))
            .collect::<Vec<Answer>>()
            .as_array()
            .unwrap()
    }

    #[must_use]
    fn read_barcode(
        imgdata: &GrayImage,
        participants_db: &HashMap<i32, Participante>,
    ) -> (Participante, OCIFase) {
        let reader = zxingcpp::read().formats([BarcodeFormat::Aztec]);
        let text;
        if let Ok(barcodes) = reader.from(imgdata)
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

            (
                // Participante encontrado na db ou default com inscrição e modalidade preenchidos.
                participants_db
                    .get(&inscricao.parse::<i32>().unwrap())
                    .cloned()
                    .unwrap_or(Participante {
                        inscricao: inscricao.to_gstring(),
                        modalidade,
                        ..Default::default()
                    }),
                fase,
            )
        } else {
            (Participante::default(), OCIFase::None)
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
        rect: &Rect,
        item_radius: u32,
        count_threshold: u32,
    ) -> [Answer; GROUP_ITEM_COUNT] {
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
    fn get_score(answers: &Answers, answer_table: AnswerTable, modalidade: OCIModalidade) -> f32 {
        if modalidade == OCIModalidade::None {
            return 0.0;
        }

        let table = match modalidade {
            OCIModalidade::IniA => answer_table.ini_a,
            OCIModalidade::IniB => answer_table.ini_b,
            OCIModalidade::Prog => answer_table.prog,
            OCIModalidade::None => unreachable!(),
        };

        let total_weight: f32 = table.iter().map(|(_, weight)| weight).sum();

        let correct_sum = table
            .iter()
            .zip(answers)
            .map(
                |((expected, weight), answer)| {
                    if answer == expected { *weight } else { 0.0 }
                },
            )
            .sum::<f32>();

        correct_sum / total_weight
    }
}
