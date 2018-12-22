#[macro_use]
extern crate web_glitz;

use web_glitz::vertex_input::Vertex;

#[derive(Vertex)] //~ ERROR: does not declare a binding location
struct VertexA {
    #[vertex_attribute]
    position: (f32, f32, f32, f32)
}
