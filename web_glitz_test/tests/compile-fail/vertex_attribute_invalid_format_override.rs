#[macro_use]
extern crate web_glitz;

use web_glitz::vertex_input::Vertex;

#[derive(Vertex)] //~ ERROR: no variant named `unknown_format`
struct VertexA {
    #[vertex_attribute(location = 0, format = "unknown_format")]
    position: (i8, i8, i8)
}
