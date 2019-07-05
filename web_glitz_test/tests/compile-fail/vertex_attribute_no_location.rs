#![feature(const_fn)]
extern crate web_glitz;

#[derive(web_glitz::derive::Vertex)] //~ ERROR: does not declare a binding location
struct VertexA {
    #[vertex_attribute(format = "Float4_f32")]
    position: [f32; 4]
}
