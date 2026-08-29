use godot::global::godot_print;
use image::GrayImage;
use rayon::prelude::*;

use crate::tools::imgtools::{AsNormal, AsRgb};

pub trait ImageFilter {
    fn apply<OP>(&mut self, op: OP) -> &mut Self
    where
        OP: Fn(&mut u8) + Sync + Send;

    fn threshold(&mut self, threshold: u8) -> &mut Self;

    fn neg(&mut self) -> &mut Self;

    fn brightness(&mut self, b: f32) -> &mut Self;

    fn gamma(&mut self, y: f32) -> &mut Self;

    fn erode(&mut self, radius: u32) -> &mut Self;

    fn dilate(&mut self, radius: u32) -> &mut Self;
}

impl ImageFilter for GrayImage {
    #[inline]
    fn apply<OP>(&mut self, op: OP) -> &mut Self
    where
        OP: Fn(&mut u8) + Sync + Send,
    {
        self.par_iter_mut().for_each(op);
        self
    }

    fn brightness(&mut self, c: f32) -> &mut Self {
        let lut: [u8; 256] = core::array::from_fn(|i| {
            let p = i as u8;
            (p.to_normal() * c).to_rgb()
        });
        self.apply(|p| *p = lut[*p as usize])
    }

    fn gamma(&mut self, y: f32) -> &mut Self {
        let lut: [u8; 256] = core::array::from_fn(|i| {
            let p = i as u8;
            p.to_normal().powf(y).to_rgb()
        });
        self.apply(|p| *p = lut[*p as usize])
    }

    fn neg(&mut self) -> &mut Self {
        self.apply(|p| *p = 255 - *p)
    }

    fn threshold(&mut self, threshold: u8) -> &mut Self {
        self.apply(|p| *p = if *p > threshold { 255 } else { 0 })
    }

    fn dilate(&mut self, radius: u32) -> &mut Self {
        let width = self.width() as usize;
        let height = self.height() as usize;
        let mut copy = self.clone();
        let radius = radius as i32;

        copy.par_chunks_mut(width).enumerate().for_each(|(y, row)| {
            for (x, p) in row.iter_mut().enumerate() {
                let mut max_neighbour = 0;

                let y_start = (y as i32 - radius).max(0) as usize;
                let y_end = (y as i32 + radius).min(height as i32 - 1) as usize;
                let x_start = (x as i32 - radius).max(0) as usize;
                let x_end = (x as i32 + radius).min(width as i32 - 1) as usize;

                for ny in y_start..=y_end {
                    let offset = ny * width;
                    for nx in x_start..=x_end {
                        let neighbour = self.as_raw()[offset + nx];
                        if neighbour > max_neighbour {
                            max_neighbour = neighbour
                        }
                    }
                }

                *p = max_neighbour
            }
        });
        *self = copy;
        self
    }

    fn erode(&mut self, radius: u32) -> &mut Self {
        let width = self.width() as usize;
        let height = self.height() as usize;
        let mut copy = self.clone();
        let radius = radius as i32;

        copy.par_chunks_mut(width).enumerate().for_each(|(y, row)| {
            for (x, p) in row.iter_mut().enumerate() {
                let mut min_neighbour = 255;

                let y_start = (y as i32 - radius).max(0) as usize;
                let y_end = (y as i32 + radius).min(height as i32 - 1) as usize;
                let x_start = (x as i32 - radius).max(0) as usize;
                let x_end = (x as i32 + radius).min(width as i32 - 1) as usize;

                for ny in y_start..=y_end {
                    let offset = ny * width;
                    for nx in x_start..=x_end {
                        let neighbour = self.as_raw()[offset + nx];
                        if neighbour < min_neighbour {
                            min_neighbour = neighbour
                        }
                    }
                }

                *p = min_neighbour
            }
        });
        *self = copy;
        self
    }
}
