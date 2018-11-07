extern crate web_glitz;

#[macro_use]
extern crate web_glitz_derive;

use web_glitz::vertex_input_binding::{AttributeFormat, Vertex, VertexInputAttributeDescriptor};

#[derive(Vertex)]
#[repr(C)]
struct VertexA {
    #[vertex_attribute(location = 0)]
    position: (f32, f32, f32, f32),
    #[vertex_attribute(location = 1)]
    normal: (f32, f32, f32),
    not_an_attribute: f32,
    #[vertex_attribute(location = 2)]
    matrix: [[f32; 4]; 4],
    #[vertex_attribute(location = 6)]
    integer: i32,
}

#[derive(Vertex)]
#[repr(C)]
struct VertexB(
    #[vertex_attribute(location = 0)] (i8, i8),
    #[vertex_attribute(location = 1)] [u8; 3],
);

#[test]
fn test_struct_attribute_descriptors() {
    let descriptors = VertexA::input_attribute_descriptors();

    assert_eq!(
        descriptors,
        vec![
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
            VertexInputAttributeDescriptor {
                location: 2,
                format: AttributeFormat::Float4x4_f32,
                offset: 32
            },
            VertexInputAttributeDescriptor {
                location: 6,
                format: AttributeFormat::Integer_i32,
                offset: 96
            },
        ]
    );
}

#[test]
fn test_tuple_struct_attribute_descriptors() {
    let descriptors = VertexB::input_attribute_descriptors();

    assert_eq!(
        descriptors,
        vec![
            VertexInputAttributeDescriptor {
                location: 0,
                format: AttributeFormat::Integer2_i8,
                offset: 0
            },
            VertexInputAttributeDescriptor {
                location: 1,
                format: AttributeFormat::Integer3_u8,
                offset: 2
            }
        ]
    );
}
