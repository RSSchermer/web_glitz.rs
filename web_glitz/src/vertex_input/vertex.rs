use crate::buffer::BufferHandle;
use crate::rendering_context::RenderingContext;

use super::{AttributeFormat, VertexInputAttributeDescriptor};

pub trait Vertex: Sized {
    type InputAttributeDescriptors: IntoIterator<Item = VertexInputAttributeDescriptor>;

    fn input_attribute_descriptors() -> Self::InputAttributeDescriptors;
}

pub trait VertexAttribute {
    fn format() -> AttributeFormat;
}

impl VertexAttribute for f32 {
    fn format() -> AttributeFormat {
        AttributeFormat::Float_f32
    }
}

impl VertexAttribute for (f32, f32) {
    fn format() -> AttributeFormat {
        AttributeFormat::Float2_f32
    }
}

impl VertexAttribute for [f32; 2] {
    fn format() -> AttributeFormat {
        AttributeFormat::Float2_f32
    }
}

impl VertexAttribute for (f32, f32, f32) {
    fn format() -> AttributeFormat {
        AttributeFormat::Float3_f32
    }
}

impl VertexAttribute for [f32; 3] {
    fn format() -> AttributeFormat {
        AttributeFormat::Float3_f32
    }
}

impl VertexAttribute for (f32, f32, f32, f32) {
    fn format() -> AttributeFormat {
        AttributeFormat::Float4_f32
    }
}

impl VertexAttribute for [f32; 4] {
    fn format() -> AttributeFormat {
        AttributeFormat::Float4_f32
    }
}

impl VertexAttribute for [[f32; 2]; 2] {
    fn format() -> AttributeFormat {
        AttributeFormat::Float2x2_f32
    }
}

impl VertexAttribute for [[f32; 2]; 3] {
    fn format() -> AttributeFormat {
        AttributeFormat::Float2x3_f32
    }
}

impl VertexAttribute for [[f32; 2]; 4] {
    fn format() -> AttributeFormat {
        AttributeFormat::Float2x4_f32
    }
}

impl VertexAttribute for [[f32; 3]; 2] {
    fn format() -> AttributeFormat {
        AttributeFormat::Float3x2_f32
    }
}

impl VertexAttribute for [[f32; 3]; 3] {
    fn format() -> AttributeFormat {
        AttributeFormat::Float3x3_f32
    }
}

impl VertexAttribute for [[f32; 3]; 4] {
    fn format() -> AttributeFormat {
        AttributeFormat::Float3x4_f32
    }
}

impl VertexAttribute for [[f32; 4]; 2] {
    fn format() -> AttributeFormat {
        AttributeFormat::Float4x2_f32
    }
}

impl VertexAttribute for [[f32; 4]; 3] {
    fn format() -> AttributeFormat {
        AttributeFormat::Float4x3_f32
    }
}

impl VertexAttribute for [[f32; 4]; 4] {
    fn format() -> AttributeFormat {
        AttributeFormat::Float4x4_f32
    }
}

impl VertexAttribute for i8 {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer_i8
    }
}

impl VertexAttribute for (i8, i8) {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer2_i8
    }
}

impl VertexAttribute for [i8; 2] {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer2_i8
    }
}

impl VertexAttribute for (i8, i8, i8) {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer3_i8
    }
}

impl VertexAttribute for [i8; 3] {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer3_i8
    }
}

impl VertexAttribute for (i8, i8, i8, i8) {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer4_i8
    }
}

impl VertexAttribute for [i8; 4] {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer4_i8
    }
}

impl VertexAttribute for i16 {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer_i16
    }
}

impl VertexAttribute for (i16, i16) {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer2_i16
    }
}

impl VertexAttribute for [i16; 2] {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer2_i16
    }
}

impl VertexAttribute for (i16, i16, i16) {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer3_i16
    }
}

impl VertexAttribute for [i16; 3] {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer3_i16
    }
}

impl VertexAttribute for (i16, i16, i16, i16) {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer4_i16
    }
}

impl VertexAttribute for [i16; 4] {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer4_i16
    }
}

impl VertexAttribute for i32 {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer_i32
    }
}

impl VertexAttribute for (i32, i32) {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer2_i32
    }
}

impl VertexAttribute for [i32; 2] {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer2_i32
    }
}

impl VertexAttribute for (i32, i32, i32) {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer3_i32
    }
}

impl VertexAttribute for [i32; 3] {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer3_i32
    }
}

impl VertexAttribute for (i32, i32, i32, i32) {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer4_i32
    }
}

impl VertexAttribute for [i32; 4] {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer4_i32
    }
}

impl VertexAttribute for u8 {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer_u8
    }
}

impl VertexAttribute for (u8, u8) {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer2_u8
    }
}

impl VertexAttribute for [u8; 2] {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer2_u8
    }
}

impl VertexAttribute for (u8, u8, u8) {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer3_u8
    }
}

impl VertexAttribute for [u8; 3] {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer3_u8
    }
}

impl VertexAttribute for (u8, u8, u8, u8) {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer4_u8
    }
}

impl VertexAttribute for [u8; 4] {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer4_u8
    }
}

impl VertexAttribute for u16 {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer_u16
    }
}

impl VertexAttribute for (u16, u16) {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer2_u16
    }
}

impl VertexAttribute for [u16; 2] {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer2_u16
    }
}

impl VertexAttribute for (u16, u16, u16) {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer3_u16
    }
}

impl VertexAttribute for [u16; 3] {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer3_u16
    }
}

impl VertexAttribute for (u16, u16, u16, u16) {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer4_u16
    }
}

impl VertexAttribute for [u16; 4] {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer4_u16
    }
}

impl VertexAttribute for u32 {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer_u32
    }
}

impl VertexAttribute for (u32, u32) {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer2_u32
    }
}

impl VertexAttribute for [u32; 2] {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer2_u32
    }
}

impl VertexAttribute for (u32, u32, u32) {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer3_u32
    }
}

impl VertexAttribute for [u32; 3] {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer3_u32
    }
}

impl VertexAttribute for (u32, u32, u32, u32) {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer4_u32
    }
}

impl VertexAttribute for [u32; 4] {
    fn format() -> AttributeFormat {
        AttributeFormat::Integer4_u32
    }
}
