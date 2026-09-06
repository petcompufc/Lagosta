use godot::prelude::*;

/// Posição relativa de uma tabela de itens no gabarito (em relação aos alinhadores)
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ItemGroup {
    pub item01a_x: f32,
    pub item01a_y: f32,
    pub item_spacing_x: f32,
    pub item_spacing_y: f32,
}

/// Retângulo de alinhamento do gabarito
#[derive(GodotClass, Clone, Copy, Default, Debug)]
#[class(init)]
pub struct Rect {
    #[var]
    pub p1: Vector2,
    #[var]
    pub p2: Vector2,
    #[var]
    pub p3: Vector2,
    #[var]
    pub p4: Vector2,
}

#[godot_api]
impl Rect {
    #[func]
    pub fn array(&self) -> Array<Vector2> {
        array![self.p1, self.p2, self.p3, self.p4]
    }

    #[func]
    pub fn angles(&self) -> Array<f32> {
        Array::from_iter(self.get_angles())
    }

    pub fn get_angles(&self) -> [f32; 4] {
        [
            (self.p1, self.p2, self.p4),
            (self.p2, self.p4, self.p3),
            (self.p4, self.p3, self.p1),
            (self.p3, self.p1, self.p2),
        ]
        .map(|points| {
            let p1 = points.0 - points.1;
            let p2 = points.2 - points.1;
            p1.angle_to(p2).to_degrees()
        })
    }
}

#[derive(GodotClass, Clone, Debug)]
#[class(init)]
pub struct ReadingParams {
    #[var]
    pub rect: Option<Gd<Rect>>,
    #[var]
    #[init(val = 3.0)]
    pub gamma: f32,
    #[var]
    #[init(val = 30)]
    pub threshold: u8,
    #[var]
    #[init(val = 7)]
    pub item_radius: u32,
    /// Threshold to count an item choice as marked when reading
    #[var]
    #[init(val = 6)]
    pub mark_threshold: u32,
    #[var]
    #[init(val = 1.5)]
    pub angle_threshold: f32,
    #[var]
    #[init(val = 26)]
    pub double_marking_threshold: u32,
}

unsafe impl Sync for ReadingParams {}
unsafe impl Send for ReadingParams {}

impl Default for ReadingParams {
    fn default() -> Self {
        Self::new_gd().bind().clone()
    }
}
