mod image_reader;
mod imgtools;
mod data;
mod barcode_writer;
use godot::prelude::*;

struct Lago;

#[gdextension]
unsafe impl ExtensionLibrary for Lago {}
