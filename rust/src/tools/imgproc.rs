use std::ops::{Deref, Range};

use image::GrayImage;
use rayon::prelude::*;

use crate::tools::imgtools::{AsNormal, AsRgb};

#[derive(Clone, Debug)]
pub struct HoughParameterSpace {
    width: u32,
    height: u32,
    space: Vec<HoughPoint>,
}

#[derive(Clone, Copy, Debug)]
pub struct HoughPoint {
    pub value: u32,
    pub rho: f32,
    pub theta: f32,
}

impl HoughParameterSpace {
    pub fn new(width: u32, height: u32, space: Vec<HoughPoint>) -> Self {
        Self {
            width,
            height,
            space,
        }
    }

    pub fn image(&self) -> GrayImage {
        let max = self.max();
        GrayImage::from_vec(
            self.width,
            self.height,
            self.space
                .par_iter()
                .map(|v| (v.value as f32 / max.value as f32).to_rgb())
                .collect(),
        )
        .unwrap()
    }

    pub fn closest_to(&self, count: u32) -> &HoughPoint {
        self.space
            .iter()
            .min_by(|a, b| (a.value.abs_diff(count)).cmp(&b.value.abs_diff(count)))
            .unwrap()
    }

    pub fn max(&self) -> &HoughPoint {
        self.space
            .iter()
            .max_by(|a, b| a.value.cmp(&b.value))
            .unwrap()
    }

    pub fn at_tr(&self, _theta: f32, _rho: f32) -> HoughPoint {
        todo!()
    }
}

impl Deref for HoughParameterSpace {
    type Target = [HoughPoint];
    fn deref(&self) -> &Self::Target {
        self.space.deref()
    }
}

impl HoughPoint {
    pub fn new(value: u32, rho: f32, theta: f32) -> Self {
        Self { value, rho, theta }
    }

    pub fn intersection_point(&self, other: &HoughPoint) -> (f32, f32) {
        let p1 = self.rho;
        let p2 = other.rho;
        let t1 = self.theta;
        let t2 = other.theta;

        let x = (p2 * t1.sin() - p1 * t2.sin()) / (t1 - t2).sin();
        let y = (p2 * t1.cos() - p1 * t2.cos()) / (t2 - t1).sin();

        (x, y)
    }
}

pub trait ImageFilter {
    fn pixelf(&self, x: u32, y: u32) -> f32;

    fn pixel(&self, x: u32, y: u32) -> u8;

    fn apply<OP>(&mut self, op: OP) -> &mut Self
    where
        OP: Fn(&mut u8) + Sync + Send;

    fn threshold(&mut self, threshold: u8) -> &mut Self;

    fn neg(&mut self) -> &mut Self;

    fn brightness(&mut self, b: f32) -> &mut Self;

    fn gamma(&mut self, y: f32) -> &mut Self;

    fn erode(&mut self, radius: u32) -> &mut Self;

    fn dilate(&mut self, radius: u32) -> &mut Self;

    fn histogram(&self) -> [u32; 256];

    fn histogram_normalization(&mut self) -> &mut Self;

    fn normalized_gradient(&mut self) -> &mut Self;

    fn derivative_x(&self, x: u32, y: u32) -> f32;

    fn derivative_y(&self, x: u32, y: u32) -> f32;

    fn hough_analysis(
        &self,
        theta_range: Range<f32>,
        theta_step: f32,
        threshold: f32,
    ) -> HoughParameterSpace;

    fn pixels_in_line(&self, theta: f32, rho: f32, threshold: f32) -> u32;
}

impl ImageFilter for GrayImage {
    #[inline]
    fn pixel(&self, x: u32, y: u32) -> u8 {
        self.as_raw()[(y * self.width() + x) as usize]
    }

    #[inline]
    fn pixelf(&self, x: u32, y: u32) -> f32 {
        self.as_raw()[(y * self.width() + x) as usize].to_normal()
    }

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
                            max_neighbour = neighbour;
                        }
                    }
                }

                *p = max_neighbour;
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
                            min_neighbour = neighbour;
                        }
                    }
                }

                *p = min_neighbour;
            }
        });
        *self = copy;
        self
    }

    fn histogram(&self) -> [u32; 256] {
        let pixel_count = self.width() * self.height();
        self.par_chunks(pixel_count as usize / rayon::max_num_threads())
            .map(|chunk| {
                let mut counter: [u32; 256] = [0; 256];
                chunk.iter().for_each(|p| counter[*p as usize] += 1);
                counter
            })
            .reduce(|| [0; 256], |a, b| std::array::from_fn(|i| a[i] + b[i]))
    }

    fn histogram_normalization(&mut self) -> &mut Self {
        let counter = self.histogram();
        let pixel_count = (self.width() * self.height()) as f32;
        let probabilities: [f32; 256] = std::array::from_fn(|i| counter[i] as f32 / pixel_count);
        let cdf: [f32; 256] = std::array::from_fn(|i| probabilities[0..=i].iter().sum());
        self.apply(|p| *p = cdf[*p as usize].to_rgb())
    }

    fn derivative_x(&self, x: u32, y: u32) -> f32 {
        if x == 0 {
            self.pixelf(1, y) - self.pixelf(0, y)
        } else if x == self.width() - 1 {
            self.pixelf(x, y) - self.pixelf(x - 1, y)
        } else {
            (self.pixelf(x + 1, y) - self.pixelf(x - 1, y)) * 0.5
        }
    }

    fn derivative_y(&self, x: u32, y: u32) -> f32 {
        if y == 0 {
            self.pixelf(x, 1) - self.pixelf(x, 0)
        } else if y == self.height() - 1 {
            self.pixelf(x, y) - self.pixelf(x, y - 1)
        } else {
            (self.pixelf(x, y + 1) - self.pixelf(x, y - 1)) * 0.5
        }
    }

    fn normalized_gradient(&mut self) -> &mut Self {
        let mut copy = self.clone();

        copy.par_iter_mut().enumerate().for_each(|(i, p)| {
            let x = i as u32 % self.width();
            let y = i as u32 / self.width();
            let dx = self.derivative_x(x, y);
            let dy = self.derivative_y(x, y);
            *p = dx.hypot(dy).to_rgb()
        });

        *self = copy;
        self
    }

    fn hough_analysis(
        &self,
        theta_range: Range<f32>,
        theta_step: f32,
        threshold: f32,
    ) -> HoughParameterSpace {
        let d = (self.width() * self.width() + self.height() * self.height()).isqrt() as i32;
        let thetas = (0..)
            .map(|i| theta_range.start + (i as f32 * theta_step))
            .take_while(|&theta| theta <= theta_range.end)
            .collect::<Vec<f32>>();

        let width = (d * 2 + 1) as u32;
        let height = thetas.len() as u32;

        let white_pixels: Vec<(u32, u32)> = (0..self.height())
            .into_par_iter()
            .map(|y| {
                let yoffset = y * self.width();
                let buf = self.as_raw();
                (0..self.width())
                    .filter_map(|x| {
                        if buf[(x + yoffset) as usize] == 255 {
                            Some((x, y))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<(u32, u32)>>()
            })
            .flatten()
            .collect();

        let space: Vec<HoughPoint> = thetas
            .into_par_iter()
            .map(|deg| {
                let theta = deg.to_radians();
                (-d..=d)
                    .map(|rho| {
                        HoughPoint::new(
                            pixels_in_line_buffer(&white_pixels, theta, rho as f32, threshold),
                            rho as f32,
                            theta,
                        )
                    })
                    .collect::<Vec<HoughPoint>>()
            })
            .flatten()
            .collect();

        HoughParameterSpace::new(width, height, space)
    }

    fn pixels_in_line(&self, theta: f32, rho: f32, threshold: f32) -> u32 {
        let sint = theta.sin();
        let cost = theta.cos();

        let mut count = 0;
        let buf = self.as_raw();

        let width = self.width() as usize;
        for y in 0..self.height() as usize {
            let y_sint = y as f32 * sint - rho;
            let y_offset = y * width;

            for x in 0..width {
                let distance_from_line = (x as f32 * cost + y_sint).abs();
                if buf[y_offset + x] == 255 && distance_from_line < threshold {
                    count += 1;
                }
            }
        }

        count
    }
}

fn pixels_in_line_buffer(buffer: &Vec<(u32, u32)>, theta: f32, rho: f32, threshold: f32) -> u32 {
    let sint = theta.sin();
    let cost = theta.cos();

    let mut count = 0;
    for pixel in buffer {
        let x = pixel.0;
        let y = pixel.1;

        let y_sint = y as f32 * sint - rho;
        let distance_from_line = (x as f32 * cost + y_sint).abs();
        if distance_from_line < threshold {
            count += 1;
        }
    }
    count
}
