#![feature(const_fn)]

use web_glitz::vertex::attribute_format::AttributeFormat;
use web_glitz::vertex::{Vertex, VertexAttributeDescriptor};

#[derive(web_glitz::derive::Vertex)]
#[repr(C)]
struct VertexA {
    #[vertex_attribute(location = 0, format = "Float4_f32")]
    position: [f32; 4],
    #[vertex_attribute(location = 1, format = "Float3_i8_norm")]
    normal: [i8; 3],
    not_an_attribute: f32,
    #[vertex_attribute(location = 2, format = "Float4x4_f32")]
    matrix: [[f32; 4]; 4],
    #[vertex_attribute(location = 6, format = "Integer_i32")]
    integer: i32,
}

#[derive(web_glitz::derive::Vertex)]
#[repr(C)]
struct VertexB(
    #[vertex_attribute(location = 0, format = "Integer2_i8")] [i8; 2],
    #[vertex_attribute(location = 1, format = "Integer3_u8")] [u8; 3],
);

#[test]
fn test_struct_attribute_descriptors() {
    let descriptors = VertexA::attribute_descriptors();

    assert_eq!(
        descriptors,
        &[
            VertexAttributeDescriptor {
                location: 0,
                format: AttributeFormat::Float4_f32,
                offset_in_bytes: 0
            },
            VertexAttributeDescriptor {
                location: 1,
                format: AttributeFormat::Float3_i8_norm,
                offset_in_bytes: 16
            },
            VertexAttributeDescriptor {
                location: 2,
                format: AttributeFormat::Float4x4_f32,
                offset_in_bytes: 24
            },
            VertexAttributeDescriptor {
                location: 6,
                format: AttributeFormat::Integer_i32,
                offset_in_bytes: 88,
            },
        ]
    );
}

#[test]
fn test_tuple_struct_attribute_descriptors() {
    let descriptors = VertexB::attribute_descriptors();

    assert_eq!(
        descriptors,
        &[
            VertexAttributeDescriptor {
                location: 0,
                format: AttributeFormat::Integer2_i8,
                offset_in_bytes: 0
            },
            VertexAttributeDescriptor {
                location: 1,
                format: AttributeFormat::Integer3_u8,
                offset_in_bytes: 2
            }
        ]
    );
}
