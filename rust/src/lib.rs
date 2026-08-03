mod sheet_reader;
mod sheet_writer;
mod data;
mod tools;

use godot::prelude::*;

struct Lago;

#[gdextension]
unsafe impl ExtensionLibrary for Lago {}
