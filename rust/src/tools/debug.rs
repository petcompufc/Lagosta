#![allow(dead_code)]

use godot::global::godot_print;

#[macro_export]
macro_rules! time {
    ($name:expr, $body:expr) => {{
        let start = std::time::Instant::now();
        let res = $body;
        godot_print!("{}: {:?}", $name, start.elapsed());
        res
    }};
}

pub struct ScopeTimer {
    name: &'static str,
    start: std::time::Instant,
}

impl ScopeTimer {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            start: std::time::Instant::now(),
        }
    }
}

impl Drop for ScopeTimer {
    fn drop(&mut self) {
        godot_print!("{}: {:?}", self.name, self.start.elapsed())
    }
}
