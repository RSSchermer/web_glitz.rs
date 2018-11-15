#[macro_use]
extern crate web_glitz;

use web_glitz::vertex_input_binding::{AttributeFormat, Vertex, VertexInputAttributeDescriptor};

#[derive(Vertex)] //~ ERROR: no variant named `unknown_format`
#[repr(C)]
struct VertexA {
    #[vertex_attribute(location = 0, format = "unknown_format")]
    position: (i8, i8, i8)
}
