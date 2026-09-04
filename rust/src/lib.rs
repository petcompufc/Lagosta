pub mod reader;
pub mod generator;
pub mod data;
pub mod tools;
// mod threader;

use godot::prelude::*;

struct Lago;

#[gdextension]
unsafe impl ExtensionLibrary for Lago {}
