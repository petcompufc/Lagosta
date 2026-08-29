pub mod sheet_reader;
pub mod answer_sheet;
pub mod data;
pub mod tools;
// mod threader;

use godot::prelude::*;

struct Lago;

#[gdextension]
unsafe impl ExtensionLibrary for Lago {}
