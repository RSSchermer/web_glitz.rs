#![feature(const_fn, const_ptr_offset_from, const_transmute, ptr_offset_from)]
extern crate web_glitz;

#[derive(web_glitz::derive::Vertex)] //~ ERROR: does not declare a binding location
struct VertexA {
    #[vertex_attribute(format = "Float4_f32")]
    position: [f32; 4]
}

fn main() {

}
