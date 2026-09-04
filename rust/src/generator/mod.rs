mod error;
use error::AnswerSheetError;

use crate::data::{OCIFase, OCIModalidade, Participante};
use base64::{Engine as _, engine::general_purpose as b64};
use godot::{
    classes::{FileAccess, Image as GDImage, ImageTexture, file_access::ModeFlags, image::Format},
    prelude::*,
};
use image::{self, ExtendedColorType, GrayAlphaImage, ImageEncoder, RgbaImage};
use krilla::{
    Document,
    geom::{Size, Transform},
    page::PageSettings,
};
use krilla_svg::{SurfaceExt, SvgSettings};
use resvg::{
    tiny_skia::Pixmap,
    usvg::{Options, Tree, fontdb::Database as FontDatabase},
};
use std::{
    path::Path,
    sync::{Arc, LazyLock},
    thread,
};

use rayon::prelude::*;

const TEMPLATE_WIDTH_PX: u32 = 793;
const TEMPLATE_HEIGHT_PX: u32 = 559;
const TEMPLATE_DEFAULT_DPI: f32 = 96.0;
const TARGET_DPI: f32 = 300.0;

const A5_WIDTH: f32 = 595.28;
const A5_HEIGHT: f32 = 419.53;

static SVG_TEMPLATE: LazyLock<String> = LazyLock::new(|| {
    // NOTE: o modo de import do arquivo precisa estar como "Keep File" na Godot
    // pra não dar problema no export final do Lagosta.
    let mut fs = FileAccess::open(
        "res://assets/templates/gabarito_automatico.svg",
        ModeFlags::READ,
    )
    .unwrap();
    let s = fs.get_as_text().to_string();
    fs.close();
    s
});

static FONT_DB: LazyLock<Arc<FontDatabase>> = LazyLock::new(|| {
    // NOTE: o modo de import do arquivo precisa estar como "Keep File" na Godot
    // pra não dar problema no export final do Lagosta.
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

/// Criador de gabaritos. Lida com a criação de gabaritos customizados.
#[derive(GodotClass)]
#[class(init)]
pub struct AnswerSheet {
    #[var]
    texture: Option<Gd<ImageTexture>>,
    #[var]
    error: GString,
}

#[godot_api]
impl AnswerSheet {
    /// Cria um handle com uma textura pra Godot.
    #[func]
    #[must_use]
    pub fn create_texture(
        participante: Gd<Participante>,
        fase: OCIFase,
        edicao: GString,
        dpi: f32,
    ) -> Gd<Self> {
        let svg_tree = match Self::new_svg(&participante.bind(), fase, &edicao.to_string(), dpi) {
            Ok(tree) => tree,
            Err(err) => {
                return Gd::from_object(Self {
                    texture: None,
                    error: err.to_string().to_gstring(),
                });
            }
        };
        let imgdata = Self::decode_svg(&svg_tree, dpi / TEMPLATE_DEFAULT_DPI);

        Gd::from_object(Self {
            texture: Some(
                ImageTexture::create_from_image(
                    &GDImage::create_from_data(
                        imgdata.width() as i32,
                        imgdata.height() as i32,
                        false,
                        Format::RGBA8,
                        &imgdata.as_ref().into(),
                    )
                    .unwrap(),
                )
                .unwrap(),
            ),
            error: "".to_gstring(),
        })
    }

    /// Checa se o gabarito gerado é válido ou se houve um erro na geração.
    #[func]
    #[inline]
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.texture.is_some()
    }

    #[func]
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn save_many(
        base_dir: GString,
        data: Array<Gd<Participante>>,
        fase: OCIFase,
        edicao: GString,
        save_bundle: bool,
        save_single: bool,
        sort_schools: bool,
        on_done: Callable,
    ) {
        // Convert to pass between threads
        let data: Vec<Participante> = data.iter_shared().map(|p| p.bind().clone()).collect();
        let edicao = &edicao.to_string();

        // Generates the SVGs from the template
        let results: Vec<Result<(Tree, Participante), AnswerSheetError>> = data
            .into_par_iter()
            .map(|part| Self::new_svg(&part, fase, edicao, TARGET_DPI).map(|tree| (tree, part)))
            .collect();
        let mut svgs = Vec::with_capacity(results.len());
        let mut errors = Vec::new();
        for result in results {
            match result {
                Ok(pair) => svgs.push(pair),
                Err(e) => errors.push(e.to_string().to_gstring()),
            }
        }

        // Saves them to PNG and PDF respectively
        let base_dir = base_dir.to_string();
        let base_path = Path::new(&base_dir);
        if sort_schools {
            // Agrupar por escolas
            svgs.sort_by(|(_, p1), (_, p2)| p1.escola.cmp(&p2.escola));
            svgs.chunk_by_mut(|(_, p1), (_, p2)| p1.escola == p2.escola)
                .for_each(|chunk| {
                    let school = chunk[0].1.escola.to_string();
                    let school_path = base_path.join(sanitize_dir_name(&school));
                    Self::handle_saving(&school_path, chunk, &mut errors, save_bundle, save_single);
                });
        } else {
            Self::handle_saving(base_path, &mut svgs, &mut errors, save_bundle, save_single);
        }

        on_done.call_deferred(&[Array::from_iter(errors).to_variant()]);
    }
}

impl AnswerSheet {
    fn handle_saving(
        path: &Path,
        svgs: &mut [(Tree, Participante)],
        error_buf: &mut Vec<GString>,
        save_bundle: bool,
        save_single: bool,
    ) {
        thread::scope(|scope| {
            let mut handles = Vec::new();
            if save_bundle {
                // Ordenar o PDF unificado por modalidade e nome
                svgs.sort_by(|a, b| {
                    a.1.modalidade
                        .cmp(&b.1.modalidade)
                        .then(a.1.nome.cmp(&b.1.nome))
                });

                handles.push(scope.spawn(|| {
                    Self::save_pdf_bundle(path, svgs)
                        .err()
                        .into_iter() // converts to vec to easily treat later
                        .collect()
                }));
            }
            if save_single {
                handles.push(
                    scope.spawn(|| Self::save_pdfs_individual(&path.join("Individuais"), svgs)),
                );
            }

            for handle in handles {
                match handle.join() {
                    Ok(errs) => error_buf.extend(errs),
                    Err(panic) => std::panic::resume_unwind(panic), // o_O
                }
            }
        });
    }

    fn save_pdf_bundle(base_path: &Path, svgs: &[(Tree, Participante)]) -> Result<(), GString> {
        // Creates the full pdf documents
        let mut document = Document::new();
        let mut a5_document = Document::new();
        let a5_size = Size::from_wh(A5_WIDTH, A5_HEIGHT).unwrap();
        let a4_size = Size::from_wh(A5_WIDTH, A5_HEIGHT * 2.0).unwrap();


        // Draws the SVGs on the PDF, two per page.
        svgs.chunks(2).for_each(|chunk| {
            let mut page = document.start_page_with(PageSettings::new(a4_size));

            let mut surface = page.surface();

            for (i, (svg_tree, _part)) in chunk.iter().enumerate() {
                let mut a5_page = a5_document.start_page_with(PageSettings::new(a5_size));
                let mut a5_surface = a5_page.surface();

                surface.push_transform(&Transform::from_translate(0.0, i as f32 * A5_HEIGHT));
                surface.draw_svg(svg_tree, a5_size, SvgSettings::default());
                surface.pop();
                a5_surface.draw_svg(svg_tree, a5_size, SvgSettings::default());

                a5_surface.finish();
                a5_page.finish();
            }

            surface.finish();
            page.finish();
        });

        // Wraps the documents
        let pdf = document
            .finish()
            .map_err(|e| format!("[Unificado] - {}", AnswerSheetError::from(e)).to_gstring())?;
        let a5_pdf = a5_document
            .finish()
            .map_err(|e| format!("[Unificado] - {}", AnswerSheetError::from(e)).to_gstring())?;

        // Writes it to disk
        std::fs::create_dir_all(base_path)
            .map_err(|e| format!("[Unificado] - {}", err_str(e)).to_gstring())?;

        let escola_sanitized = sanitize_dir_name(&svgs[0].1.escola.to_string());
        let file_path = base_path.join(format!("{escola_sanitized}.pdf"));
        let a5_file_path = base_path.join(format!("A5_{escola_sanitized}.pdf"));

        std::fs::write(file_path, &pdf)
            .map_err(|e| format!("[Unificado] - {}", err_str(e)).to_gstring())?;
        std::fs::write(a5_file_path, &a5_pdf)
            .map_err(|e| format!("[Unificado] - {}", err_str(e)).to_gstring())?;
        Ok(())
    }

    #[must_use]
    fn save_pdfs_individual(base_path: &Path, svgs: &[(Tree, Participante)]) -> Vec<GString> {
        // this is ugly but, meh. it works. and i'm tired.
        let err = std::fs::create_dir_all(base_path.join(OCIModalidade::IniA.to_string()));
        if let Err(e) = err {
            return vec![format!("Erro criando diretório: {e:?}").to_gstring()];
        };
        let err = std::fs::create_dir_all(base_path.join(OCIModalidade::IniB.to_string()));
        if let Err(e) = err {
            return vec![format!("Erro criando diretório: {e:?}").to_gstring()];
        };
        let err = std::fs::create_dir_all(base_path.join(OCIModalidade::Prog.to_string()));
        if let Err(e) = err {
            return vec![format!("Erro criando diretório: {e:?}").to_gstring()];
        };

        svgs.par_iter()
            .map(|(svg_tree, part)| {
                // Writes svg to new A5 PDF
                let mut document = Document::new();
                let svg_size = Size::from_wh(A5_WIDTH, A5_HEIGHT).unwrap();
                let mut page = document.start_page_with(PageSettings::new(svg_size));
                let mut surface = page.surface();
                surface.draw_svg(svg_tree, svg_size, SvgSettings::default());
                surface.finish();
                page.finish();

                // Wraps the document and writes it to disk
                let pdf = document
                    .finish()
                    .map_err(|e| (AnswerSheetError::from(e), part))?;

                let file_name = format!("{}_{}", part.nome, part.inscricao);
                let file_path = base_path
                    .join(part.modalidade.to_string())
                    .join(format!("{}.pdf", sanitize_dir_name(&file_name)));
                std::fs::write(file_path, &pdf).map_err(|e| (AnswerSheetError::from(e), part))?;
                Ok(())
            })
            .filter_map(Result::err)
            .map(|(e, part)| format!("[Insc. {}] - {}", part.inscricao, e).to_gstring())
            .collect()
    }

    fn new_svg(
        participante: &Participante,
        fase: OCIFase,
        edicao: &str,
        dpi: f32,
    ) -> Result<Tree, AnswerSheetError> {
        // Formato do código de barras: mf00000000
        // m: letra da modalidade (a, b, p)
        // f: número da fase (1, 2, 3)
        let barcode_str = format!(
            "{}{}{}",
            participante.modalidade.char(),
            fase.char(),
            participante.inscricao
        );

        // Cria a imagem do código de barras e encoda como um png em base64 pra colocar no svg
        let barcode_img = Self::create_barcode(barcode_str)?;
        let barcode_base64 = Self::grayalpha_to_base64(&barcode_img)?;

        // Insere as informações numa cópia do template SVG
        let svg_str = SVG_TEMPLATE
            .replace("{BARCODE}", barcode_base64.as_str())
            .replace("{PARTICIPANTE}", &participante.nome.to_string())
            .replace("{ESCOLA}", &participante.escola.to_string())
            .replace("{MODALIDADE}", &participante.modalidade.to_string())
            .replace("{FASE}", &fase.to_string())
            .replace("{EDICAO}", edicao)
            .replace("{INSCRICAO}", &participante.inscricao.to_string());

        let svg_options = Options {
            dpi,
            fontdb: FONT_DB.clone(),
            ..Default::default()
        };

        Tree::from_str(&svg_str, &svg_options).map_err(AnswerSheetError::from)
    }

    /// Encoda uma string em um buffer `LumaA8` (grayscale + alpha)
    fn create_barcode(text: String) -> Result<GrayAlphaImage, AnswerSheetError> {
        let aztec = zxingcpp::create(zxingcpp::BarcodeFormat::Aztec)
            .from_str(text)?
            .to_image_with(&zxingcpp::write().scale(5))?;
        let (width, height) = (aztec.width() as u32, aztec.height() as u32);

        // Converts the Luma8 image into LumaA8
        let data_size = (width * height) as usize;
        let imgdata: Vec<u16> = aztec
            .data()
            .into_iter()
            .map(|pixel| {
                // Makes white pixels transparent: pixels will be read as two bytes: alpha|luma.
                // we invert it so black becomes 1|0 and white becomes 0|0.
                // (the bitshift is due to little-endianness)
                !(u16::from(pixel)) << 8
            })
            .collect();

        // Reinterpret the Vec<u16> as Vec<u8> with double the size.
        let (imgdata_ptr, _, cap) = imgdata.into_raw_parts();
        let imgdata_ptr = imgdata_ptr.cast::<u8>();
        let imgdata_u8 = unsafe { Vec::from_raw_parts(imgdata_ptr, data_size * 2, cap * 2) };

        Ok(GrayAlphaImage::from_raw(width, height, imgdata_u8).unwrap())
    }

    /// Decoda uma string svg em uma buffer RGBA. \
    /// Usa RGBA pq é o padrão do `Pixmap` que o `resvg` usa.
    fn decode_svg(svg_tree: &Tree, scale: f32) -> RgbaImage {
        let (width, height) = (
            (TEMPLATE_WIDTH_PX as f32 * scale).floor() as u32,
            (TEMPLATE_HEIGHT_PX as f32 * scale).floor() as u32,
        );
        let mut pix = Pixmap::new(width, height).unwrap();

        resvg::render(
            svg_tree,
            resvg::usvg::Transform::default(),
            &mut pix.as_mut(),
        );

        RgbaImage::from_raw(width, height, pix.take()).unwrap()
    }

    /// Encoda um buffer de pixels `LumaA8` em um PNG Base64
    fn grayalpha_to_base64(img: &GrayAlphaImage) -> Result<String, AnswerSheetError> {
        let mut pngvec = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut pngvec);
        encoder.write_image(
            img.as_ref(),
            img.width(),
            img.height(),
            ExtendedColorType::La8,
        )?;
        Ok(b64::STANDARD.encode(pngvec))
    }
}

pub fn sanitize_dir_name(input: &str) -> String {
    let forbidden = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

    let sanitized: String = input
        .replace(" ", "_")
        .chars()
        .filter(|c| !c.is_control())
        .map(|c| if forbidden.contains(&c) { '_' } else { c })
        .collect();

    let trimmed: String = sanitized
        .trim_end_matches([' ', '.'])
        .chars()
        .take(50)
        .collect();
    if trimmed.is_empty() {
        "escola_sem_nome".to_string()
    } else {
        trimmed
    }
}

#[inline]
#[must_use]
fn err_str<E>(err: E) -> String
where
    AnswerSheetError: From<E>,
{
    AnswerSheetError::from(err).to_string()
}
