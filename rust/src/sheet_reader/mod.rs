mod reading;

use crate::tools::imgtools::*;
use godot::{
    classes::{Image as GDImage, ImageTexture, image::Format},
    prelude::*,
};
use imageproc::{
    contrast::{self, ThresholdType},
    distance_transform::Norm,
    image::{self, GrayImage},
    morphology,
};
use zxingcpp::BarcodeFormat;

#[derive(GodotClass)]
#[class(base=Node, init, tool)]
struct SheetReader {
    #[export(global_file = "*.png,*.jpg,*.jpeg,*.bmp,*.webp")]
    #[var(set = load_image)]
    image_path: GString,

    #[export]
    #[init(val = true)]
    barcode: bool,

    imgdata: Option<GrayImage>,
    processed_imgdata: Option<GrayImage>,

    base: Base<Node>,
}

#[godot_api]
impl SheetReader {
    #[signal]
    fn image_loaded();
    #[signal]
    fn image_processed();

    /// Loads a new image into the reader.
    #[func]
    fn load_image(&mut self, image_path: GString) {
        self.imgdata = image::open(image_path.to_string())
            .ok()
            .map(|img| img.into_luma8());

        if self.imgdata.is_none() && !image_path.is_empty() {
            godot_error!("Couldn't open image {image_path}.");
        }

        self.image_path = image_path;
        self.signals().image_loaded().emit();
    }

    /// Clones image data and applies filters to it
    #[func]
    fn process_image(&mut self) {
        if let Some(mut imgdata) = self.imgdata.clone() {
            apply_filter(imgdata.as_mut(), |p| 1.0 - p.powf(1.5));
            contrast::threshold_mut(&mut imgdata, 60, ThresholdType::Binary);
            morphology::erode_mut(&mut imgdata, Norm::LInf, 2);
            morphology::dilate_mut(&mut imgdata, Norm::LInf, 2);

            self.processed_imgdata = Some(imgdata);
            self.signals().image_processed().emit();
        };
    }

    #[func]
    fn read_barcode(&mut self) -> GString {
        if let Some(imgdata) = self.imgdata.as_ref() {
            let reader = zxingcpp::read()
                .formats(&[BarcodeFormat::Aztec])
                .try_invert(false);
            let barcodes = reader.from(imgdata).expect("eita porra");
            return barcodes.first().unwrap().text().as_str().into();
        }
        "".into()
    }

    #[func]
    #[inline]
    fn create_texture_original(&self) -> Option<Gd<ImageTexture>> {
        Self::create_godot_texture(self.imgdata.as_ref()?)
    }

    #[func]
    #[inline]
    fn create_texture_processed(&self) -> Option<Gd<ImageTexture>> {
        Self::create_godot_texture(self.processed_imgdata.as_ref()?)
    }

    fn create_godot_texture(imgdata: &GrayImage) -> Option<Gd<ImageTexture>> {
        let godot_image = GDImage::create_from_data(
            imgdata.width() as i32,
            imgdata.height() as i32,
            false,
            Format::L8,
            &imgdata.as_ref().into(),
        )?;
        let i = ImageTexture::create_from_image(&godot_image);
        i
    }
}
