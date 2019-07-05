#![feature(const_fn)]
extern crate web_glitz;

#[derive(web_glitz::derive::Vertex)] //~ ERROR: cannot find type `unknown_format`
struct VertexA {
    #[vertex_attribute(location = 0, format = "unknown_format")]
    position: [i8; 3]
}
