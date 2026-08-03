mod sheet_reader;
mod answer_sheet;
mod data;
mod tools;

use godot::prelude::*;

struct Lago;

#[gdextension]
unsafe impl ExtensionLibrary for Lago {}
