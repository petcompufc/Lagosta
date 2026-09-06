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
}

unsafe impl Sync for ReadingParams {}
unsafe impl Send for ReadingParams {}


impl Default for ReadingParams {
    fn default() -> Self {
        Self::new_gd().bind().clone()
    }
}
