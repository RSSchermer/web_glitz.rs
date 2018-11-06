extern crate web_glitz;

#[macro_use]
extern crate web_glitz_derive;

use web_glitz::vertex_input_binding::{ Vertex, VertexInputAttributeDescriptor, AttributeFormat };

#[derive(Vertex)]
#[repr(C)]
struct VertexA {
    #[vertex_attribute(location = 0)]
    position: (f32, f32, f32, f32),
    #[vertex_attribute(location = 1)]
    normal: (f32, f32, f32),
    not_an_attribute: f32
}

#[test]
fn test_vertex_a_attribute_descriptors() {
    let descriptors = VertexA::input_attribute_descriptors();

    assert_eq!(descriptors, vec![
        VertexInputAttributeDescriptor {
            location: 0,
            format: AttributeFormat::Float4_f32,
            offset: 0
        },
        VertexInputAttributeDescriptor {
            location: 1,
            format: AttributeFormat::Float3_f32,
            offset: 16
        },
    ]);
}