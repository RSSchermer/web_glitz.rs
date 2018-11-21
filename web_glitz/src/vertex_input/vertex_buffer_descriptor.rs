pub trait VertexBufferDescriptor {
    fn stride_in_bytes(&self) -> u8;

    fn input_rate(&self) -> InputRate;

    fn attributes(&self) -> &[VertexInputAttributeDescriptor];
}

pub enum InputRate {
    PerVertex,
    PerInstance,
}

#[derive(PartialEq, Debug)]
pub struct VertexInputAttributeDescriptor {
    pub location: u32,
    pub offset: u8,
    pub format: AttributeFormat,
}

#[derive(PartialEq, Debug)]
#[allow(non_camel_case_types)]
pub enum AttributeFormat {
    Float_f32,
    Float_i8_fixed,
    Float_i8_norm,
    Float_i16_fixed,
    Float_i16_norm,
    Float_u8_fixed,
    Float_u8_norm,
    Float_u16_fixed,
    Float_u16_norm,
    Float2_f32,
    Float2_i8_fixed,
    Float2_i8_norm,
    Float2_i16_fixed,
    Float2_i16_norm,
    Float2_u8_fixed,
    Float2_u8_norm,
    Float2_u16_fixed,
    Float2_u16_norm,
    Float3_f32,
    Float3_i8_fixed,
    Float3_i8_norm,
    Float3_i16_fixed,
    Float3_i16_norm,
    Float3_u8_fixed,
    Float3_u8_norm,
    Float3_u16_fixed,
    Float3_u16_norm,
    Float4_f32,
    Float4_i8_fixed,
    Float4_i8_norm,
    Float4_i16_fixed,
    Float4_i16_norm,
    Float4_u8_fixed,
    Float4_u8_norm,
    Float4_u16_fixed,
    Float4_u16_norm,
    Float2x2_f32,
    Float2x2_i8_fixed,
    Float2x2_i8_norm,
    Float2x2_i16_fixed,
    Float2x2_i16_norm,
    Float2x2_u8_fixed,
    Float2x2_u8_norm,
    Float2x2_u16_fixed,
    Float2x2_u16_norm,
    Float2x3_f32,
    Float2x3_i8_fixed,
    Float2x3_i8_norm,
    Float2x3_i16_fixed,
    Float2x3_i16_norm,
    Float2x3_u8_fixed,
    Float2x3_u8_norm,
    Float2x3_u16_fixed,
    Float2x3_u16_norm,
    Float2x4_f32,
    Float2x4_i8_fixed,
    Float2x4_i8_norm,
    Float2x4_i16_fixed,
    Float2x4_i16_norm,
    Float2x4_u8_fixed,
    Float2x4_u8_norm,
    Float2x4_u16_fixed,
    Float2x4_u16_norm,
    Float3x2_f32,
    Float3x2_i8_fixed,
    Float3x2_i8_norm,
    Float3x2_i16_fixed,
    Float3x2_i16_norm,
    Float3x2_u8_fixed,
    Float3x2_u8_norm,
    Float3x2_u16_fixed,
    Float3x2_u16_norm,
    Float3x3_f32,
    Float3x3_i8_fixed,
    Float3x3_i8_norm,
    Float3x3_i16_fixed,
    Float3x3_i16_norm,
    Float3x3_u8_fixed,
    Float3x3_u8_norm,
    Float3x3_u16_fixed,
    Float3x3_u16_norm,
    Float3x4_f32,
    Float3x4_i8_fixed,
    Float3x4_i8_norm,
    Float3x4_i16_fixed,
    Float3x4_i16_norm,
    Float3x4_u8_fixed,
    Float3x4_u8_norm,
    Float3x4_u16_fixed,
    Float3x4_u16_norm,
    Float4x2_f32,
    Float4x2_i8_fixed,
    Float4x2_i8_norm,
    Float4x2_i16_fixed,
    Float4x2_i16_norm,
    Float4x2_u8_fixed,
    Float4x2_u8_norm,
    Float4x2_u16_fixed,
    Float4x2_u16_norm,
    Float4x3_f32,
    Float4x3_i8_fixed,
    Float4x3_i8_norm,
    Float4x3_i16_fixed,
    Float4x3_i16_norm,
    Float4x3_u8_fixed,
    Float4x3_u8_norm,
    Float4x3_u16_fixed,
    Float4x3_u16_norm,
    Float4x4_f32,
    Float4x4_i8_fixed,
    Float4x4_i8_norm,
    Float4x4_i16_fixed,
    Float4x4_i16_norm,
    Float4x4_u8_fixed,
    Float4x4_u8_norm,
    Float4x4_u16_fixed,
    Float4x4_u16_norm,
    Integer_i8,
    Integer_u8,
    Integer_i16,
    Integer_u16,
    Integer_i32,
    Integer_u32,
    Integer2_i8,
    Integer2_u8,
    Integer2_i16,
    Integer2_u16,
    Integer2_i32,
    Integer2_u32,
    Integer3_i8,
    Integer3_u8,
    Integer3_i16,
    Integer3_u16,
    Integer3_i32,
    Integer3_u32,
    Integer4_i8,
    Integer4_u8,
    Integer4_i16,
    Integer4_u16,
    Integer4_i32,
    Integer4_u32,
}