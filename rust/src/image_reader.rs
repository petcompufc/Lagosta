use godot::prelude::*;

#[derive(GodotClass)]
#[class(base=Node, init)]
struct ImageReader {

}

#[godot_api]
impl ImageReader {
    #[func]
    fn print() {
        godot_print!("Hello, World!")
    }
}
