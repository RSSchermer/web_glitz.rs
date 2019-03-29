use super::attribute_format::*;
use super::VertexInputAttributeDescriptor;

pub unsafe trait Vertex: Sized {
    fn input_attribute_descriptors() -> &'static [VertexInputAttributeDescriptor];
}

pub unsafe trait FormatCompatible<F>
where
    F: AttributeFormat,
{
}

unsafe impl FormatCompatible<Float_f32> for f32 {}
unsafe impl FormatCompatible<Float_i8_fixed> for i8 {}
unsafe impl FormatCompatible<Float_i8_norm> for i8 {}
unsafe impl FormatCompatible<Float_u8_fixed> for u8 {}
unsafe impl FormatCompatible<Float_u8_norm> for u8 {}
unsafe impl FormatCompatible<Float_i16_fixed> for i16 {}
unsafe impl FormatCompatible<Float_i16_norm> for i16 {}
unsafe impl FormatCompatible<Float_u16_fixed> for u16 {}
unsafe impl FormatCompatible<Float_u16_norm> for u16 {}
unsafe impl FormatCompatible<Float2_f32> for [f32; 2] {}
unsafe impl FormatCompatible<Float2_i8_fixed> for [i8; 2] {}
unsafe impl FormatCompatible<Float2_i8_norm> for [i8; 2] {}
unsafe impl FormatCompatible<Float2_u8_fixed> for [u8; 2] {}
unsafe impl FormatCompatible<Float2_u8_norm> for [u8; 2] {}
unsafe impl FormatCompatible<Float2_i16_fixed> for [i16; 2] {}
unsafe impl FormatCompatible<Float2_i16_norm> for [i16; 2] {}
unsafe impl FormatCompatible<Float2_u16_fixed> for [u16; 2] {}
unsafe impl FormatCompatible<Float2_u16_norm> for [u16; 2] {}
unsafe impl FormatCompatible<Float3_f32> for [f32; 3] {}
unsafe impl FormatCompatible<Float3_i8_fixed> for [i8; 3] {}
unsafe impl FormatCompatible<Float3_i8_norm> for [i8; 3] {}
unsafe impl FormatCompatible<Float3_u8_fixed> for [u8; 3] {}
unsafe impl FormatCompatible<Float3_u8_norm> for [u8; 3] {}
unsafe impl FormatCompatible<Float3_i16_fixed> for [i16; 3] {}
unsafe impl FormatCompatible<Float3_i16_norm> for [i16; 3] {}
unsafe impl FormatCompatible<Float3_u16_fixed> for [u16; 3] {}
unsafe impl FormatCompatible<Float3_u16_norm> for [u16; 3] {}
unsafe impl FormatCompatible<Float4_f32> for [f32; 4] {}
unsafe impl FormatCompatible<Float4_i8_fixed> for [i8; 4] {}
unsafe impl FormatCompatible<Float4_i8_norm> for [i8; 4] {}
unsafe impl FormatCompatible<Float4_u8_fixed> for [u8; 4] {}
unsafe impl FormatCompatible<Float4_u8_norm> for [u8; 4] {}
unsafe impl FormatCompatible<Float4_i16_fixed> for [i16; 4] {}
unsafe impl FormatCompatible<Float4_i16_norm> for [i16; 4] {}
unsafe impl FormatCompatible<Float4_u16_fixed> for [u16; 4] {}
unsafe impl FormatCompatible<Float4_u16_norm> for [u16; 4] {}
unsafe impl FormatCompatible<Float2x2_f32> for [[f32; 2]; 2] {}
unsafe impl FormatCompatible<Float2x2_i8_fixed> for [[i8; 2]; 2] {}
unsafe impl FormatCompatible<Float2x2_i8_norm> for [[i8; 2]; 2] {}
unsafe impl FormatCompatible<Float2x2_u8_fixed> for [[u8; 2]; 2] {}
unsafe impl FormatCompatible<Float2x2_u8_norm> for [[u8; 2]; 2] {}
unsafe impl FormatCompatible<Float2x2_i16_fixed> for [[i16; 2]; 2] {}
unsafe impl FormatCompatible<Float2x2_i16_norm> for [[i16; 2]; 2] {}
unsafe impl FormatCompatible<Float2x2_u16_fixed> for [[u16; 2]; 2] {}
unsafe impl FormatCompatible<Float2x2_u16_norm> for [[u16; 2]; 2] {}
unsafe impl FormatCompatible<Float2x3_f32> for [[f32; 3]; 2] {}
unsafe impl FormatCompatible<Float2x3_i8_fixed> for [[i8; 3]; 2] {}
unsafe impl FormatCompatible<Float2x3_i8_norm> for [[i8; 3]; 2] {}
unsafe impl FormatCompatible<Float2x3_u8_fixed> for [[u8; 3]; 2] {}
unsafe impl FormatCompatible<Float2x3_u8_norm> for [[u8; 3]; 2] {}
unsafe impl FormatCompatible<Float2x3_i16_fixed> for [[i16; 3]; 2] {}
unsafe impl FormatCompatible<Float2x3_i16_norm> for [[i16; 3]; 2] {}
unsafe impl FormatCompatible<Float2x3_u16_fixed> for [[u16; 3]; 2] {}
unsafe impl FormatCompatible<Float2x3_u16_norm> for [[u16; 3]; 2] {}
unsafe impl FormatCompatible<Float2x4_f32> for [[f32; 4]; 2] {}
unsafe impl FormatCompatible<Float2x4_i8_fixed> for [[i8; 4]; 2] {}
unsafe impl FormatCompatible<Float2x4_i8_norm> for [[i8; 4]; 2] {}
unsafe impl FormatCompatible<Float2x4_u8_fixed> for [[u8; 4]; 2] {}
unsafe impl FormatCompatible<Float2x4_u8_norm> for [[u8; 4]; 2] {}
unsafe impl FormatCompatible<Float2x4_i16_fixed> for [[i16; 4]; 2] {}
unsafe impl FormatCompatible<Float2x4_i16_norm> for [[i16; 4]; 2] {}
unsafe impl FormatCompatible<Float2x4_u16_fixed> for [[u16; 4]; 2] {}
unsafe impl FormatCompatible<Float2x4_u16_norm> for [[u16; 4]; 2] {}
unsafe impl FormatCompatible<Float3x2_f32> for [[f32; 2]; 3] {}
unsafe impl FormatCompatible<Float3x2_i8_fixed> for [[i8; 2]; 3] {}
unsafe impl FormatCompatible<Float3x2_i8_norm> for [[i8; 2]; 3] {}
unsafe impl FormatCompatible<Float3x2_u8_fixed> for [[u8; 2]; 3] {}
unsafe impl FormatCompatible<Float3x2_u8_norm> for [[u8; 2]; 3] {}
unsafe impl FormatCompatible<Float3x2_i16_fixed> for [[i16; 2]; 3] {}
unsafe impl FormatCompatible<Float3x2_i16_norm> for [[i16; 2]; 3] {}
unsafe impl FormatCompatible<Float3x2_u16_fixed> for [[u16; 2]; 3] {}
unsafe impl FormatCompatible<Float3x2_u16_norm> for [[u16; 2]; 3] {}
unsafe impl FormatCompatible<Float3x3_f32> for [[f32; 3]; 3] {}
unsafe impl FormatCompatible<Float3x3_i8_fixed> for [[i8; 3]; 3] {}
unsafe impl FormatCompatible<Float3x3_i8_norm> for [[i8; 3]; 3] {}
unsafe impl FormatCompatible<Float3x3_u8_fixed> for [[u8; 3]; 3] {}
unsafe impl FormatCompatible<Float3x3_u8_norm> for [[u8; 3]; 3] {}
unsafe impl FormatCompatible<Float3x3_i16_fixed> for [[i16; 3]; 3] {}
unsafe impl FormatCompatible<Float3x3_i16_norm> for [[i16; 3]; 3] {}
unsafe impl FormatCompatible<Float3x3_u16_fixed> for [[u16; 3]; 3] {}
unsafe impl FormatCompatible<Float3x3_u16_norm> for [[u16; 3]; 3] {}
unsafe impl FormatCompatible<Float3x4_f32> for [[f32; 4]; 3] {}
unsafe impl FormatCompatible<Float3x4_i8_fixed> for [[i8; 4]; 3] {}
unsafe impl FormatCompatible<Float3x4_i8_norm> for [[i8; 4]; 3] {}
unsafe impl FormatCompatible<Float3x4_u8_fixed> for [[u8; 4]; 3] {}
unsafe impl FormatCompatible<Float3x4_u8_norm> for [[u8; 4]; 3] {}
unsafe impl FormatCompatible<Float3x4_i16_fixed> for [[i16; 4]; 3] {}
unsafe impl FormatCompatible<Float3x4_i16_norm> for [[i16; 4]; 3] {}
unsafe impl FormatCompatible<Float3x4_u16_fixed> for [[u16; 4]; 3] {}
unsafe impl FormatCompatible<Float3x4_u16_norm> for [[u16; 4]; 3] {}
unsafe impl FormatCompatible<Float4x2_f32> for [[f32; 2]; 4] {}
unsafe impl FormatCompatible<Float4x2_i8_fixed> for [[i8; 2]; 4] {}
unsafe impl FormatCompatible<Float4x2_i8_norm> for [[i8; 2]; 4] {}
unsafe impl FormatCompatible<Float4x2_u8_fixed> for [[u8; 2]; 4] {}
unsafe impl FormatCompatible<Float4x2_u8_norm> for [[u8; 2]; 4] {}
unsafe impl FormatCompatible<Float4x2_i16_fixed> for [[i16; 2]; 4] {}
unsafe impl FormatCompatible<Float4x2_i16_norm> for [[i16; 2]; 4] {}
unsafe impl FormatCompatible<Float4x2_u16_fixed> for [[u16; 2]; 4] {}
unsafe impl FormatCompatible<Float4x2_u16_norm> for [[u16; 2]; 4] {}
unsafe impl FormatCompatible<Float4x3_f32> for [[f32; 3]; 4] {}
unsafe impl FormatCompatible<Float4x3_i8_fixed> for [[i8; 3]; 4] {}
unsafe impl FormatCompatible<Float4x3_i8_norm> for [[i8; 3]; 4] {}
unsafe impl FormatCompatible<Float4x3_u8_fixed> for [[u8; 3]; 4] {}
unsafe impl FormatCompatible<Float4x3_u8_norm> for [[u8; 3]; 4] {}
unsafe impl FormatCompatible<Float4x3_i16_fixed> for [[i16; 3]; 4] {}
unsafe impl FormatCompatible<Float4x3_i16_norm> for [[i16; 3]; 4] {}
unsafe impl FormatCompatible<Float4x3_u16_fixed> for [[u16; 3]; 4] {}
unsafe impl FormatCompatible<Float4x3_u16_norm> for [[u16; 3]; 4] {}
unsafe impl FormatCompatible<Float4x4_f32> for [[f32; 4]; 4] {}
unsafe impl FormatCompatible<Float4x4_i8_fixed> for [[i8; 4]; 4] {}
unsafe impl FormatCompatible<Float4x4_i8_norm> for [[i8; 4]; 4] {}
unsafe impl FormatCompatible<Float4x4_u8_fixed> for [[u8; 4]; 4] {}
unsafe impl FormatCompatible<Float4x4_u8_norm> for [[u8; 4]; 4] {}
unsafe impl FormatCompatible<Float4x4_i16_fixed> for [[i16; 4]; 4] {}
unsafe impl FormatCompatible<Float4x4_i16_norm> for [[i16; 4]; 4] {}
unsafe impl FormatCompatible<Float4x4_u16_fixed> for [[u16; 4]; 4] {}
unsafe impl FormatCompatible<Float4x4_u16_norm> for [[u16; 4]; 4] {}
unsafe impl FormatCompatible<Integer_i8> for i8 {}
unsafe impl FormatCompatible<Integer_i16> for i16 {}
unsafe impl FormatCompatible<Integer_i32> for i32 {}
unsafe impl FormatCompatible<Integer_u8> for u8 {}
unsafe impl FormatCompatible<Integer_u16> for u16 {}
unsafe impl FormatCompatible<Integer_u32> for u32 {}
unsafe impl FormatCompatible<Integer2_i8> for [i8; 2] {}
unsafe impl FormatCompatible<Integer2_i16> for [i16; 2] {}
unsafe impl FormatCompatible<Integer2_i32> for [i32; 2] {}
unsafe impl FormatCompatible<Integer2_u8> for [u8; 2] {}
unsafe impl FormatCompatible<Integer2_u16> for [u16; 2] {}
unsafe impl FormatCompatible<Integer2_u32> for [u32; 2] {}
unsafe impl FormatCompatible<Integer3_i8> for [i8; 3] {}
unsafe impl FormatCompatible<Integer3_i16> for [i16; 3] {}
unsafe impl FormatCompatible<Integer3_i32> for [i32; 3] {}
unsafe impl FormatCompatible<Integer3_u8> for [u8; 3] {}
unsafe impl FormatCompatible<Integer3_u16> for [u16; 3] {}
unsafe impl FormatCompatible<Integer3_u32> for [u32; 3] {}
unsafe impl FormatCompatible<Integer4_i8> for [i8; 4] {}
unsafe impl FormatCompatible<Integer4_i16> for [i16; 4] {}
unsafe impl FormatCompatible<Integer4_i32> for [i32; 4] {}
unsafe impl FormatCompatible<Integer4_u8> for [u8; 4] {}
unsafe impl FormatCompatible<Integer4_u16> for [u16; 4] {}
unsafe impl FormatCompatible<Integer4_u32> for [u32; 4] {}
