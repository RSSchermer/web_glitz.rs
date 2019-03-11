#![allow(non_camel_case_types)]

use crate::pipeline::graphics::vertex_input::FormatKind;

pub unsafe trait AttributeFormat {
    fn kind() -> FormatKind;
}

pub struct Float_f32;

unsafe impl AttributeFormat for Float_f32 {
    fn kind() -> FormatKind {
        FormatKind::Float_f32
    }
}

pub struct Float_i8_fixed;

unsafe impl AttributeFormat for Float_i8_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float_i8_fixed
    }
}

pub struct Float_i8_norm;

unsafe impl AttributeFormat for Float_i8_norm {
    fn kind() -> FormatKind {
        FormatKind::Float_i8_norm
    }
}

pub struct Float_i16_fixed;

unsafe impl AttributeFormat for Float_i16_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float_i16_fixed
    }
}

pub struct Float_i16_norm;

unsafe impl AttributeFormat for Float_i16_norm {
    fn kind() -> FormatKind {
        FormatKind::Float_i16_norm
    }
}

pub struct Float_u8_fixed;

unsafe impl AttributeFormat for Float_u8_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float_u8_fixed
    }
}

pub struct Float_u8_norm;

unsafe impl AttributeFormat for Float_u8_norm {
    fn kind() -> FormatKind {
        FormatKind::Float_u8_norm
    }
}

pub struct Float_u16_fixed;

unsafe impl AttributeFormat for Float_u16_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float_u16_fixed
    }
}

pub struct Float_u16_norm;

unsafe impl AttributeFormat for Float_u16_norm {
    fn kind() -> FormatKind {
        FormatKind::Float_u16_norm
    }
}

pub struct Float2_f32;

unsafe impl AttributeFormat for Float2_f32 {
    fn kind() -> FormatKind {
        FormatKind::Float2_f32
    }
}

pub struct Float2_i8_fixed;

unsafe impl AttributeFormat for Float2_i8_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float2_i8_fixed
    }
}

pub struct Float2_i8_norm;

unsafe impl AttributeFormat for Float2_i8_norm {
    fn kind() -> FormatKind {
        FormatKind::Float2_i8_norm
    }
}

pub struct Float2_i16_fixed;

unsafe impl AttributeFormat for Float2_i16_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float2_i16_fixed
    }
}

pub struct Float2_i16_norm;

unsafe impl AttributeFormat for Float2_i16_norm {
    fn kind() -> FormatKind {
        FormatKind::Float2_i16_norm
    }
}

pub struct Float2_u8_fixed;

unsafe impl AttributeFormat for Float2_u8_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float2_u8_fixed
    }
}

pub struct Float2_u8_norm;

unsafe impl AttributeFormat for Float2_u8_norm {
    fn kind() -> FormatKind {
        FormatKind::Float2_u8_norm
    }
}

pub struct Float2_u16_fixed;

unsafe impl AttributeFormat for Float2_u16_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float2_u16_fixed
    }
}

pub struct Float2_u16_norm;

unsafe impl AttributeFormat for Float2_u16_norm {
    fn kind() -> FormatKind {
        FormatKind::Float2_u16_norm
    }
}

pub struct Float3_f32;

unsafe impl AttributeFormat for Float3_f32 {
    fn kind() -> FormatKind {
        FormatKind::Float3_f32
    }
}

pub struct Float3_i8_fixed;

unsafe impl AttributeFormat for Float3_i8_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float3_i8_fixed
    }
}

pub struct Float3_i8_norm;

unsafe impl AttributeFormat for Float3_i8_norm {
    fn kind() -> FormatKind {
        FormatKind::Float3_i8_norm
    }
}

pub struct Float3_i16_fixed;

unsafe impl AttributeFormat for Float3_i16_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float3_i16_fixed
    }
}

pub struct Float3_i16_norm;

unsafe impl AttributeFormat for Float3_i16_norm {
    fn kind() -> FormatKind {
        FormatKind::Float3_i16_norm
    }
}

pub struct Float3_u8_fixed;

unsafe impl AttributeFormat for Float3_u8_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float3_u8_fixed
    }
}

pub struct Float3_u8_norm;

unsafe impl AttributeFormat for Float3_u8_norm {
    fn kind() -> FormatKind {
        FormatKind::Float3_u8_norm
    }
}

pub struct Float3_u16_fixed;

unsafe impl AttributeFormat for Float3_u16_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float3_u16_fixed
    }
}

pub struct Float3_u16_norm;

unsafe impl AttributeFormat for Float3_u16_norm {
    fn kind() -> FormatKind {
        FormatKind::Float3_u16_norm
    }
}

pub struct Float4_f32;

unsafe impl AttributeFormat for Float4_f32 {
    fn kind() -> FormatKind {
        FormatKind::Float4_f32
    }
}

pub struct Float4_i8_fixed;

unsafe impl AttributeFormat for Float4_i8_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float4_i8_fixed
    }
}

pub struct Float4_i8_norm;

unsafe impl AttributeFormat for Float4_i8_norm {
    fn kind() -> FormatKind {
        FormatKind::Float4_i8_norm
    }
}

pub struct Float4_i16_fixed;

unsafe impl AttributeFormat for Float4_i16_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float4_i16_fixed
    }
}

pub struct Float4_i16_norm;

unsafe impl AttributeFormat for Float4_i16_norm {
    fn kind() -> FormatKind {
        FormatKind::Float4_i16_norm
    }
}

pub struct Float4_u8_fixed;

unsafe impl AttributeFormat for Float4_u8_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float4_u8_fixed
    }
}

pub struct Float4_u8_norm;

unsafe impl AttributeFormat for Float4_u8_norm {
    fn kind() -> FormatKind {
        FormatKind::Float4_u8_norm
    }
}

pub struct Float4_u16_fixed;

unsafe impl AttributeFormat for Float4_u16_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float4_u16_fixed
    }
}

pub struct Float4_u16_norm;

unsafe impl AttributeFormat for Float4_u16_norm {
    fn kind() -> FormatKind {
        FormatKind::Float4_u16_norm
    }
}

pub struct Float2x2_f32;

unsafe impl AttributeFormat for Float2x2_f32 {
    fn kind() -> FormatKind {
        FormatKind::Float2x2_f32
    }
}

pub struct Float2x2_i8_fixed;

unsafe impl AttributeFormat for Float2x2_i8_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float2x2_i8_fixed
    }
}

pub struct Float2x2_i8_norm;

unsafe impl AttributeFormat for Float2x2_i8_norm {
    fn kind() -> FormatKind {
        FormatKind::Float2x2_i8_norm
    }
}

pub struct Float2x2_i16_fixed;

unsafe impl AttributeFormat for Float2x2_i16_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float2x2_i16_fixed
    }
}

pub struct Float2x2_i16_norm;

unsafe impl AttributeFormat for Float2x2_i16_norm {
    fn kind() -> FormatKind {
        FormatKind::Float2x2_i16_norm
    }
}

pub struct Float2x2_u8_fixed;

unsafe impl AttributeFormat for Float2x2_u8_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float2x2_u8_fixed
    }
}

pub struct Float2x2_u8_norm;

unsafe impl AttributeFormat for Float2x2_u8_norm {
    fn kind() -> FormatKind {
        FormatKind::Float2x2_u8_norm
    }
}

pub struct Float2x2_u16_fixed;

unsafe impl AttributeFormat for Float2x2_u16_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float2x2_u16_fixed
    }
}

pub struct Float2x2_u16_norm;

unsafe impl AttributeFormat for Float2x2_u16_norm {
    fn kind() -> FormatKind {
        FormatKind::Float2x2_u16_norm
    }
}

pub struct Float2x3_f32;

unsafe impl AttributeFormat for Float2x3_f32 {
    fn kind() -> FormatKind {
        FormatKind::Float2x3_f32
    }
}

pub struct Float2x3_i8_fixed;

unsafe impl AttributeFormat for Float2x3_i8_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float2x3_i8_fixed
    }
}

pub struct Float2x3_i8_norm;

unsafe impl AttributeFormat for Float2x3_i8_norm {
    fn kind() -> FormatKind {
        FormatKind::Float2x3_i8_norm
    }
}

pub struct Float2x3_i16_fixed;

unsafe impl AttributeFormat for Float2x3_i16_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float2x3_i16_fixed
    }
}

pub struct Float2x3_i16_norm;

unsafe impl AttributeFormat for Float2x3_i16_norm {
    fn kind() -> FormatKind {
        FormatKind::Float2x3_i16_norm
    }
}

pub struct Float2x3_u8_fixed;

unsafe impl AttributeFormat for Float2x3_u8_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float2x3_u8_fixed
    }
}

pub struct Float2x3_u8_norm;

unsafe impl AttributeFormat for Float2x3_u8_norm {
    fn kind() -> FormatKind {
        FormatKind::Float2x3_u8_norm
    }
}

pub struct Float2x3_u16_fixed;

unsafe impl AttributeFormat for Float2x3_u16_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float2x3_u16_fixed
    }
}

pub struct Float2x3_u16_norm;

unsafe impl AttributeFormat for Float2x3_u16_norm {
    fn kind() -> FormatKind {
        FormatKind::Float2x3_u16_norm
    }
}

pub struct Float2x4_f32;

unsafe impl AttributeFormat for Float2x4_f32 {
    fn kind() -> FormatKind {
        FormatKind::Float2x4_f32
    }
}

pub struct Float2x4_i8_fixed;

unsafe impl AttributeFormat for Float2x4_i8_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float2x4_i8_fixed
    }
}

pub struct Float2x4_i8_norm;

unsafe impl AttributeFormat for Float2x4_i8_norm {
    fn kind() -> FormatKind {
        FormatKind::Float2x4_i8_norm
    }
}

pub struct Float2x4_i16_fixed;

unsafe impl AttributeFormat for Float2x4_i16_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float2x4_i16_fixed
    }
}

pub struct Float2x4_i16_norm;

unsafe impl AttributeFormat for Float2x4_i16_norm {
    fn kind() -> FormatKind {
        FormatKind::Float2x4_i16_norm
    }
}

pub struct Float2x4_u8_fixed;

unsafe impl AttributeFormat for Float2x4_u8_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float2x4_u8_fixed
    }
}

pub struct Float2x4_u8_norm;

unsafe impl AttributeFormat for Float2x4_u8_norm {
    fn kind() -> FormatKind {
        FormatKind::Float2x4_u8_norm
    }
}

pub struct Float2x4_u16_fixed;

unsafe impl AttributeFormat for Float2x4_u16_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float2x4_u16_fixed
    }
}

pub struct Float2x4_u16_norm;

unsafe impl AttributeFormat for Float2x4_u16_norm {
    fn kind() -> FormatKind {
        FormatKind::Float2x4_u16_norm
    }
}

pub struct Float3x2_f32;

unsafe impl AttributeFormat for Float3x2_f32 {
    fn kind() -> FormatKind {
        FormatKind::Float3x2_f32
    }
}

pub struct Float3x2_i8_fixed;

unsafe impl AttributeFormat for Float3x2_i8_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float3x2_i8_fixed
    }
}

pub struct Float3x2_i8_norm;

unsafe impl AttributeFormat for Float3x2_i8_norm {
    fn kind() -> FormatKind {
        FormatKind::Float3x2_i8_norm
    }
}

pub struct Float3x2_i16_fixed;

unsafe impl AttributeFormat for Float3x2_i16_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float3x2_i16_fixed
    }
}

pub struct Float3x2_i16_norm;

unsafe impl AttributeFormat for Float3x2_i16_norm {
    fn kind() -> FormatKind {
        FormatKind::Float3x2_i16_norm
    }
}

pub struct Float3x2_u8_fixed;

unsafe impl AttributeFormat for Float3x2_u8_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float3x2_u8_fixed
    }
}

pub struct Float3x2_u8_norm;

unsafe impl AttributeFormat for Float3x2_u8_norm {
    fn kind() -> FormatKind {
        FormatKind::Float3x2_u8_norm
    }
}

pub struct Float3x2_u16_fixed;

unsafe impl AttributeFormat for Float3x2_u16_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float3x2_u16_fixed
    }
}

pub struct Float3x2_u16_norm;

unsafe impl AttributeFormat for Float3x2_u16_norm {
    fn kind() -> FormatKind {
        FormatKind::Float3x2_u16_norm
    }
}

pub struct Float3x3_f32;

unsafe impl AttributeFormat for Float3x3_f32 {
    fn kind() -> FormatKind {
        FormatKind::Float3x3_f32
    }
}

pub struct Float3x3_i8_fixed;

unsafe impl AttributeFormat for Float3x3_i8_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float3x3_i8_fixed
    }
}

pub struct Float3x3_i8_norm;

unsafe impl AttributeFormat for Float3x3_i8_norm {
    fn kind() -> FormatKind {
        FormatKind::Float3x3_i8_norm
    }
}

pub struct Float3x3_i16_fixed;

unsafe impl AttributeFormat for Float3x3_i16_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float3x3_i16_fixed
    }
}

pub struct Float3x3_i16_norm;

unsafe impl AttributeFormat for Float3x3_i16_norm {
    fn kind() -> FormatKind {
        FormatKind::Float3x3_i16_norm
    }
}

pub struct Float3x3_u8_fixed;

unsafe impl AttributeFormat for Float3x3_u8_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float3x3_u8_fixed
    }
}

pub struct Float3x3_u8_norm;

unsafe impl AttributeFormat for Float3x3_u8_norm {
    fn kind() -> FormatKind {
        FormatKind::Float3x3_u8_norm
    }
}

pub struct Float3x3_u16_fixed;

unsafe impl AttributeFormat for Float3x3_u16_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float3x3_u16_fixed
    }
}

pub struct Float3x3_u16_norm;

unsafe impl AttributeFormat for Float3x3_u16_norm {
    fn kind() -> FormatKind {
        FormatKind::Float3x3_u16_norm
    }
}

pub struct Float3x4_f32;

unsafe impl AttributeFormat for Float3x4_f32 {
    fn kind() -> FormatKind {
        FormatKind::Float3x4_f32
    }
}

pub struct Float3x4_i8_fixed;

unsafe impl AttributeFormat for Float3x4_i8_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float3x4_i8_fixed
    }
}

pub struct Float3x4_i8_norm;

unsafe impl AttributeFormat for Float3x4_i8_norm {
    fn kind() -> FormatKind {
        FormatKind::Float3x4_i8_norm
    }
}

pub struct Float3x4_i16_fixed;

unsafe impl AttributeFormat for Float3x4_i16_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float3x4_i16_fixed
    }
}

pub struct Float3x4_i16_norm;

unsafe impl AttributeFormat for Float3x4_i16_norm {
    fn kind() -> FormatKind {
        FormatKind::Float3x4_i16_norm
    }
}

pub struct Float3x4_u8_fixed;

unsafe impl AttributeFormat for Float3x4_u8_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float3x4_u8_fixed
    }
}

pub struct Float3x4_u8_norm;

unsafe impl AttributeFormat for Float3x4_u8_norm {
    fn kind() -> FormatKind {
        FormatKind::Float3x4_u8_norm
    }
}

pub struct Float3x4_u16_fixed;

unsafe impl AttributeFormat for Float3x4_u16_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float3x4_u16_fixed
    }
}

pub struct Float3x4_u16_norm;

unsafe impl AttributeFormat for Float3x4_u16_norm {
    fn kind() -> FormatKind {
        FormatKind::Float3x4_u16_norm
    }
}

pub struct Float4x2_f32;

unsafe impl AttributeFormat for Float4x2_f32 {
    fn kind() -> FormatKind {
        FormatKind::Float4x2_f32
    }
}

pub struct Float4x2_i8_fixed;

unsafe impl AttributeFormat for Float4x2_i8_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float4x2_i8_fixed
    }
}

pub struct Float4x2_i8_norm;

unsafe impl AttributeFormat for Float4x2_i8_norm {
    fn kind() -> FormatKind {
        FormatKind::Float4x2_i8_norm
    }
}

pub struct Float4x2_i16_fixed;

unsafe impl AttributeFormat for Float4x2_i16_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float4x2_i16_fixed
    }
}

pub struct Float4x2_i16_norm;

unsafe impl AttributeFormat for Float4x2_i16_norm {
    fn kind() -> FormatKind {
        FormatKind::Float4x2_i16_norm
    }
}

pub struct Float4x2_u8_fixed;

unsafe impl AttributeFormat for Float4x2_u8_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float4x2_u8_fixed
    }
}

pub struct Float4x2_u8_norm;

unsafe impl AttributeFormat for Float4x2_u8_norm {
    fn kind() -> FormatKind {
        FormatKind::Float4x2_u8_norm
    }
}

pub struct Float4x2_u16_fixed;

unsafe impl AttributeFormat for Float4x2_u16_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float4x2_u16_fixed
    }
}

pub struct Float4x2_u16_norm;

unsafe impl AttributeFormat for Float4x2_u16_norm {
    fn kind() -> FormatKind {
        FormatKind::Float4x2_u16_norm
    }
}

pub struct Float4x3_f32;

unsafe impl AttributeFormat for Float4x3_f32 {
    fn kind() -> FormatKind {
        FormatKind::Float4x3_f32
    }
}

pub struct Float4x3_i8_fixed;

unsafe impl AttributeFormat for Float4x3_i8_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float4x3_i8_fixed
    }
}

pub struct Float4x3_i8_norm;

unsafe impl AttributeFormat for Float4x3_i8_norm {
    fn kind() -> FormatKind {
        FormatKind::Float4x3_i8_norm
    }
}

pub struct Float4x3_i16_fixed;

unsafe impl AttributeFormat for Float4x3_i16_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float4x3_i16_fixed
    }
}

pub struct Float4x3_i16_norm;

unsafe impl AttributeFormat for Float4x3_i16_norm {
    fn kind() -> FormatKind {
        FormatKind::Float4x3_i16_norm
    }
}

pub struct Float4x3_u8_fixed;

unsafe impl AttributeFormat for Float4x3_u8_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float4x3_u8_fixed
    }
}

pub struct Float4x3_u8_norm;

unsafe impl AttributeFormat for Float4x3_u8_norm {
    fn kind() -> FormatKind {
        FormatKind::Float4x3_u8_norm
    }
}

pub struct Float4x3_u16_fixed;

unsafe impl AttributeFormat for Float4x3_u16_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float4x3_u16_fixed
    }
}

pub struct Float4x3_u16_norm;

unsafe impl AttributeFormat for Float4x3_u16_norm {
    fn kind() -> FormatKind {
        FormatKind::Float4x3_u16_norm
    }
}

pub struct Float4x4_f32;

unsafe impl AttributeFormat for Float4x4_f32 {
    fn kind() -> FormatKind {
        FormatKind::Float4x4_f32
    }
}

pub struct Float4x4_i8_fixed;

unsafe impl AttributeFormat for Float4x4_i8_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float4x4_i8_fixed
    }
}

pub struct Float4x4_i8_norm;

unsafe impl AttributeFormat for Float4x4_i8_norm {
    fn kind() -> FormatKind {
        FormatKind::Float4x4_i8_norm
    }
}

pub struct Float4x4_i16_fixed;

unsafe impl AttributeFormat for Float4x4_i16_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float4x4_i16_fixed
    }
}

pub struct Float4x4_i16_norm;

unsafe impl AttributeFormat for Float4x4_i16_norm {
    fn kind() -> FormatKind {
        FormatKind::Float4x4_i16_norm
    }
}

pub struct Float4x4_u8_fixed;

unsafe impl AttributeFormat for Float4x4_u8_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float4x4_u8_fixed
    }
}

pub struct Float4x4_u8_norm;

unsafe impl AttributeFormat for Float4x4_u8_norm {
    fn kind() -> FormatKind {
        FormatKind::Float4x4_u8_norm
    }
}

pub struct Float4x4_u16_fixed;

unsafe impl AttributeFormat for Float4x4_u16_fixed {
    fn kind() -> FormatKind {
        FormatKind::Float4x4_u16_fixed
    }
}

pub struct Float4x4_u16_norm;

unsafe impl AttributeFormat for Float4x4_u16_norm {
    fn kind() -> FormatKind {
        FormatKind::Float4x4_u16_norm
    }
}

pub struct Integer_i8;

unsafe impl AttributeFormat for Integer_i8 {
    fn kind() -> FormatKind {
        FormatKind::Integer_i8
    }
}

pub struct Integer_u8;

unsafe impl AttributeFormat for Integer_u8 {
    fn kind() -> FormatKind {
        FormatKind::Integer_u8
    }
}

pub struct Integer_i16;

unsafe impl AttributeFormat for Integer_i16 {
    fn kind() -> FormatKind {
        FormatKind::Integer_i16
    }
}

pub struct Integer_u16;

unsafe impl AttributeFormat for Integer_u16 {
    fn kind() -> FormatKind {
        FormatKind::Integer_u16
    }
}

pub struct Integer_i32;

unsafe impl AttributeFormat for Integer_i32 {
    fn kind() -> FormatKind {
        FormatKind::Integer_i32
    }
}

pub struct Integer_u32;

unsafe impl AttributeFormat for Integer_u32 {
    fn kind() -> FormatKind {
        FormatKind::Integer_u32
    }
}

pub struct Integer2_i8;

unsafe impl AttributeFormat for Integer2_i8 {
    fn kind() -> FormatKind {
        FormatKind::Integer2_i8
    }
}

pub struct Integer2_u8;

unsafe impl AttributeFormat for Integer2_u8 {
    fn kind() -> FormatKind {
        FormatKind::Integer2_u8
    }
}

pub struct Integer2_i16;

unsafe impl AttributeFormat for Integer2_i16 {
    fn kind() -> FormatKind {
        FormatKind::Integer2_i16
    }
}

pub struct Integer2_u16;

unsafe impl AttributeFormat for Integer2_u16 {
    fn kind() -> FormatKind {
        FormatKind::Integer2_u16
    }
}

pub struct Integer2_i32;

unsafe impl AttributeFormat for Integer2_i32 {
    fn kind() -> FormatKind {
        FormatKind::Integer2_i32
    }
}

pub struct Integer2_u32;

unsafe impl AttributeFormat for Integer2_u32 {
    fn kind() -> FormatKind {
        FormatKind::Integer2_u32
    }
}

pub struct Integer3_i8;

unsafe impl AttributeFormat for Integer3_i8 {
    fn kind() -> FormatKind {
        FormatKind::Integer3_i8
    }
}

pub struct Integer3_u8;

unsafe impl AttributeFormat for Integer3_u8 {
    fn kind() -> FormatKind {
        FormatKind::Integer3_u8
    }
}

pub struct Integer3_i16;

unsafe impl AttributeFormat for Integer3_i16 {
    fn kind() -> FormatKind {
        FormatKind::Integer3_i16
    }
}

pub struct Integer3_u16;

unsafe impl AttributeFormat for Integer3_u16 {
    fn kind() -> FormatKind {
        FormatKind::Integer3_u16
    }
}

pub struct Integer3_i32;

unsafe impl AttributeFormat for Integer3_i32 {
    fn kind() -> FormatKind {
        FormatKind::Integer3_i32
    }
}

pub struct Integer3_u32;

unsafe impl AttributeFormat for Integer3_u32 {
    fn kind() -> FormatKind {
        FormatKind::Integer3_u32
    }
}

pub struct Integer4_i8;

unsafe impl AttributeFormat for Integer4_i8 {
    fn kind() -> FormatKind {
        FormatKind::Integer4_i8
    }
}

pub struct Integer4_u8;

unsafe impl AttributeFormat for Integer4_u8 {
    fn kind() -> FormatKind {
        FormatKind::Integer4_u8
    }
}

pub struct Integer4_i16;

unsafe impl AttributeFormat for Integer4_i16 {
    fn kind() -> FormatKind {
        FormatKind::Integer4_i16
    }
}

pub struct Integer4_u16;

unsafe impl AttributeFormat for Integer4_u16 {
    fn kind() -> FormatKind {
        FormatKind::Integer4_u16
    }
}

pub struct Integer4_i32;

unsafe impl AttributeFormat for Integer4_i32 {
    fn kind() -> FormatKind {
        FormatKind::Integer4_i32
    }
}

pub struct Integer4_u32;

unsafe impl AttributeFormat for Integer4_u32 {
    fn kind() -> FormatKind {
        FormatKind::Integer4_u32
    }
}
