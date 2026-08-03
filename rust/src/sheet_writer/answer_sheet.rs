use godot::{
    classes::{Image as GDImage, ImageTexture, image::Format},
    prelude::*,
};
use imageproc::image::RgbaImage;

#[derive(GodotConvert, Var, Export, Default, Clone, Debug, Copy)]
#[godot(via=GString)]
pub enum AnswerSheetError {
    #[default]
    IniA,
    IniB,
    Prog,
}

/// Uma handle pros dados gerados pelo `SheetWriter`.
#[derive(GodotClass)]
#[class(init)]
pub struct AnswerSheet {
    imgdata: Option<RgbaImage>,
    #[var]
    errors: Array<AnswerSheetError>,
}

#[godot_api]
impl AnswerSheet {
    pub fn new(imgdata: Option<RgbaImage>, errors: Array<AnswerSheetError>) -> Gd<Self> {
        Gd::from_object(Self { imgdata, errors })
    }

    #[func]
    fn create_texture(&self) -> Gd<ImageTexture> {
        let imgdata = self
            .imgdata
            .as_ref()
            .expect("Tried creating texture from invalid answersheet.");
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
        .unwrap()
    }

    #[func]
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.imgdata.is_some()
    }
}
