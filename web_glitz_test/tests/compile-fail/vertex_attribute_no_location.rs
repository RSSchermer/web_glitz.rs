#[macro_use]
extern crate web_glitz;

use web_glitz::vertex_input_binding::{AttributeFormat, Vertex, VertexInputAttributeDescriptor};

#[derive(Vertex)] //~ ERROR: does not declare a binding location
#[repr(C)]
struct VertexA {
    #[vertex_attribute]
    position: (f32, f32, f32, f32)
}
