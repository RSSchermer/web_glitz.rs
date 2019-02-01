#[macro_use]
extern crate web_glitz;

use web_glitz::vertex_input::Vertex;

#[derive(Vertex)]
struct VertexA {
    #[vertex_attribute(location = 0)]
    position: String //~ ERROR: the trait bound `std::string::String: web_glitz::vertex_input::VertexAttribute` is not satisfied
}
