#![feature(const_fn)]
extern crate web_glitz;

#[derive(web_glitz::derive::Vertex)]
struct VertexA {
    #[vertex_attribute(location = 0, format = "Float4_f32")] //~ ERROR: the trait bound `std::string::String: web_glitz::vertex::attribute_format::FormatCompatible<web_glitz::vertex::attribute_format::Float4_f32>` is not satisfied
    position: String
}
