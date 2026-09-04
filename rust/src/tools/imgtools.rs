#![allow(dead_code)]

use godot::classes::{Image, ImageTexture, image::Format};
use godot::prelude::*;
use image::{GrayAlphaImage, GrayImage, RgbImage, imageops};
use rayon::prelude::*;

pub fn apply_filter<F>(pixels: &mut [u8], mut filter: F)
where
    F: FnMut(f32) -> f32,
{
    // Lookup table: Runs the filter once for all 256 possible values.
    let lut = std::array::from_fn::<u8, 256, _>(|i| {
        let val = filter(i as f32 / 255.0) * 255.0;
        val.clamp(0.0, 255.0) as u8
    });

    // Applies the filter for every pixel
    for p in pixels {
        *p = lut[*p as usize];
    }
}

pub trait AsNormal {
    fn to_normal(self) -> f32;
    fn as_normal<F>(&self, action: F) -> Self
    where
        F: FnOnce(f32) -> f32;
    fn as_normal_mut<F>(&mut self, action: F)
    where
        F: FnOnce(f32) -> f32;
}

impl AsNormal for u8 {
    #[inline]
    fn to_normal(self) -> f32 {
        f32::from(self) / 255.0
    }

    #[inline]
    fn as_normal<F>(&self, action: F) -> Self
    where
        F: FnOnce(f32) -> f32,
    {
        action(self.to_normal()).to_rgb()
    }

    #[inline]
    fn as_normal_mut<F>(&mut self, action: F)
    where
        F: FnOnce(f32) -> f32,
    {
        *self = self.as_normal(action);
    }
}

pub trait AsRgb {
    fn to_rgb(self) -> u8;
    fn as_rgb<F>(&self, action: F) -> Self
    where
        F: FnOnce(u8) -> u8;
    fn as_rgb_mut<F>(&mut self, action: F)
    where
        F: FnOnce(u8) -> u8;
}

impl AsRgb for f32 {
    #[inline]
    fn to_rgb(self) -> u8 {
        (self * 255.0) as u8
    }

    #[inline]
    fn as_rgb<F>(&self, action: F) -> Self
    where
        F: FnOnce(u8) -> u8,
    {
        action(self.to_rgb()).to_normal()
    }

    #[inline]
    fn as_rgb_mut<F>(&mut self, action: F)
    where
        F: FnOnce(u8) -> u8,
    {
        *self = self.as_rgb(action);
    }
}

pub fn create_godot_texture(imgdata: &GrayImage) -> Option<Gd<ImageTexture>> {
    let godot_image = Image::create_from_data(
        imgdata.width() as i32,
        imgdata.height() as i32,
        false,
        Format::L8,
        &imgdata.as_ref().into(),
    )?;
    ImageTexture::create_from_image(&godot_image)
}

pub fn create_godot_color_texture(imgdata: &RgbImage) -> Option<Gd<ImageTexture>> {
    let godot_image = Image::create_from_data(
        imgdata.width() as i32,
        imgdata.height() as i32,
        false,
        Format::RGB8,
        &imgdata.as_ref().into(),
    )?;
    ImageTexture::create_from_image(&godot_image)
}

#[must_use]
pub fn fit_image_to(image: &GrayImage, target_width: u32, target_height: u32) -> GrayImage {
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

pub fn draw_sqr(image: &mut GrayImage, x: u32, y: u32, radius: i32) {
    let width = image.width();
    for i in -radius..radius {
        for j in -radius..radius {
            let dy = (y as i32 + j).clamp(0, image.height() as i32 - 1) as usize;
            let dx = (x as i32 + i).clamp(0, image.width() as i32 - 1) as usize;
            image.as_mut()[dy * width as usize + dx] = 128
        }
    }
}

pub fn clear_transparent(image: &mut GrayAlphaImage) {
    image.as_mut().par_chunks_mut(2).for_each(|p| {
        if p[1] < 255 {
            p[0] = 255;
            p[1] = 255;
        }
    });
}
