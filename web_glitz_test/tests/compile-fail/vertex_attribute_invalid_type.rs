#[macro_use]
extern crate web_glitz;

use web_glitz::vertex_input::Vertex;

// NOTE: I don't understand why this error does not appear on the correct line here, it seems to do
// so everywhere else...

#[derive(Vertex)] //~ ERROR: the trait bound `std::string::String: web_glitz::vertex_input::VertexAttribute` is not satisfied
struct VertexA {
    #[vertex_attribute(location = 0)]
    position: String
}
