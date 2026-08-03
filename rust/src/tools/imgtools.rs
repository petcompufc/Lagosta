#![allow(dead_code)]
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
    pixels.iter_mut().for_each(|p| *p = lut[*p as usize]);
}

pub trait AsNormal {
    fn to_normal(self) -> f32;
    fn as_normal<F>(self, action: F) -> Self
    where
        F: FnOnce(f32) -> f32;
    fn as_normal_mut<F>(&mut self, action: F)
    where
        F: FnOnce(f32) -> f32;
}

impl AsNormal for u8 {
    #[inline]
    fn to_normal(self) -> f32 {
        self as f32 / 255.0
    }

    #[inline]
    fn as_normal<F>(self, action: F) -> Self
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
        *self = self.as_normal(action)
    }
}

pub trait AsRgb {
    fn to_rgb(self) -> u8;
    fn as_rgb<F>(self, action: F) -> Self
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
    fn as_rgb<F>(self, action: F) -> Self
    where
        F: FnOnce(u8) -> u8,
    {
        action(self.to_rgb()).to_normal() as f32
    }

    #[inline]
    fn as_rgb_mut<F>(&mut self, action: F)
    where
        F: FnOnce(u8) -> u8,
    {
        *self = self.as_rgb(action)
    }
}
