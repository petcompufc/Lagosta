mod data;
mod image_reader;
mod imgtools;
mod sheet_writer;
mod debug;

use godot::prelude::*;

struct Lago;

#[gdextension]
unsafe impl ExtensionLibrary for Lago {}
