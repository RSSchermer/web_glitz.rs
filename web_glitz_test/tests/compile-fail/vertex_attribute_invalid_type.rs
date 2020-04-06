#![feature(const_fn, const_ptr_offset_from, const_transmute, ptr_offset_from)]
extern crate web_glitz;

#[derive(web_glitz::derive::Vertex)]
struct VertexA {
    #[vertex_attribute(location = 0, format = "Float4_f32")]
    position: String //~ ERROR: the trait bound `std::string::String: web_glitz::pipeline::graphics::attribute_format::VertexAttributeFormatCompatible<web_glitz::pipeline::graphics::attribute_format::Float4_f32>` is not satisfied
}

fn main() {

}

