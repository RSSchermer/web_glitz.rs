#![allow(non_camel_case_types)]

use crate::pipeline::graphics::VertexAttributeType;

/// Trait implemented by attribute format identifiers.
///
/// Helper trait which, in conjunction with [FormatCompatible], allows the derive macro for the
/// [Vertex] trait to verify at compile time that an attribute field type is compatible with the
/// specified attribute format.
pub trait VertexAttributeFormatIdentifier {
    /// The [AttributeFormat] associated with this [AttributeFormatIdentifier].
    const FORMAT: VertexAttributeFormat;
}

pub struct Float_f32;

impl VertexAttributeFormatIdentifier for Float_f32 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float_f32;
}

pub struct Float_i8_fixed;

impl VertexAttributeFormatIdentifier for Float_i8_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float_i8_fixed;
}

pub struct Float_i8_norm;

impl VertexAttributeFormatIdentifier for Float_i8_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float_i8_norm;
}

pub struct Float_i16_fixed;

impl VertexAttributeFormatIdentifier for Float_i16_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float_i16_fixed;
}

pub struct Float_i16_norm;

impl VertexAttributeFormatIdentifier for Float_i16_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float_i16_norm;
}

pub struct Float_u8_fixed;

impl VertexAttributeFormatIdentifier for Float_u8_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float_u8_fixed;
}

pub struct Float_u8_norm;

impl VertexAttributeFormatIdentifier for Float_u8_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float_u8_norm;
}

pub struct Float_u16_fixed;

impl VertexAttributeFormatIdentifier for Float_u16_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float_u16_fixed;
}

pub struct Float_u16_norm;

impl VertexAttributeFormatIdentifier for Float_u16_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float_u16_norm;
}

pub struct Float2_f32;

impl VertexAttributeFormatIdentifier for Float2_f32 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2_f32;
}

pub struct Float2_i8_fixed;

impl VertexAttributeFormatIdentifier for Float2_i8_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2_i8_fixed;
}

pub struct Float2_i8_norm;

impl VertexAttributeFormatIdentifier for Float2_i8_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2_i8_norm;
}

pub struct Float2_i16_fixed;

impl VertexAttributeFormatIdentifier for Float2_i16_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2_i16_fixed;
}

pub struct Float2_i16_norm;

impl VertexAttributeFormatIdentifier for Float2_i16_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2_i16_norm;
}

pub struct Float2_u8_fixed;

impl VertexAttributeFormatIdentifier for Float2_u8_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2_u8_fixed;
}

pub struct Float2_u8_norm;

impl VertexAttributeFormatIdentifier for Float2_u8_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2_u8_norm;
}

pub struct Float2_u16_fixed;

impl VertexAttributeFormatIdentifier for Float2_u16_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2_u16_fixed;
}

pub struct Float2_u16_norm;

impl VertexAttributeFormatIdentifier for Float2_u16_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2_u16_norm;
}

pub struct Float3_f32;

impl VertexAttributeFormatIdentifier for Float3_f32 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3_f32;
}

pub struct Float3_i8_fixed;

impl VertexAttributeFormatIdentifier for Float3_i8_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3_i8_fixed;
}

pub struct Float3_i8_norm;

impl VertexAttributeFormatIdentifier for Float3_i8_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3_i8_norm;
}

pub struct Float3_i16_fixed;

impl VertexAttributeFormatIdentifier for Float3_i16_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3_i16_fixed;
}

pub struct Float3_i16_norm;

impl VertexAttributeFormatIdentifier for Float3_i16_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3_i16_norm;
}

pub struct Float3_u8_fixed;

impl VertexAttributeFormatIdentifier for Float3_u8_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3_u8_fixed;
}

pub struct Float3_u8_norm;

impl VertexAttributeFormatIdentifier for Float3_u8_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3_u8_norm;
}

pub struct Float3_u16_fixed;

impl VertexAttributeFormatIdentifier for Float3_u16_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3_u16_fixed;
}

pub struct Float3_u16_norm;

impl VertexAttributeFormatIdentifier for Float3_u16_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3_u16_norm;
}

pub struct Float4_f32;

impl VertexAttributeFormatIdentifier for Float4_f32 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4_f32;
}

pub struct Float4_i8_fixed;

impl VertexAttributeFormatIdentifier for Float4_i8_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4_i8_fixed;
}

pub struct Float4_i8_norm;

impl VertexAttributeFormatIdentifier for Float4_i8_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4_i8_norm;
}

pub struct Float4_i16_fixed;

impl VertexAttributeFormatIdentifier for Float4_i16_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4_i16_fixed;
}

pub struct Float4_i16_norm;

impl VertexAttributeFormatIdentifier for Float4_i16_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4_i16_norm;
}

pub struct Float4_u8_fixed;

impl VertexAttributeFormatIdentifier for Float4_u8_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4_u8_fixed;
}

pub struct Float4_u8_norm;

impl VertexAttributeFormatIdentifier for Float4_u8_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4_u8_norm;
}

pub struct Float4_u16_fixed;

impl VertexAttributeFormatIdentifier for Float4_u16_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4_u16_fixed;
}

pub struct Float4_u16_norm;

impl VertexAttributeFormatIdentifier for Float4_u16_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4_u16_norm;
}

pub struct Float2x2_f32;

impl VertexAttributeFormatIdentifier for Float2x2_f32 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2x2_f32;
}

pub struct Float2x2_i8_fixed;

impl VertexAttributeFormatIdentifier for Float2x2_i8_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2x2_i8_fixed;
}

pub struct Float2x2_i8_norm;

impl VertexAttributeFormatIdentifier for Float2x2_i8_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2x2_i8_norm;
}

pub struct Float2x2_i16_fixed;

impl VertexAttributeFormatIdentifier for Float2x2_i16_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2x2_i16_fixed;
}

pub struct Float2x2_i16_norm;

impl VertexAttributeFormatIdentifier for Float2x2_i16_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2x2_i16_norm;
}

pub struct Float2x2_u8_fixed;

impl VertexAttributeFormatIdentifier for Float2x2_u8_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2x2_u8_fixed;
}

pub struct Float2x2_u8_norm;

impl VertexAttributeFormatIdentifier for Float2x2_u8_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2x2_u8_norm;
}

pub struct Float2x2_u16_fixed;

impl VertexAttributeFormatIdentifier for Float2x2_u16_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2x2_u16_fixed;
}

pub struct Float2x2_u16_norm;

impl VertexAttributeFormatIdentifier for Float2x2_u16_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2x2_u16_norm;
}

pub struct Float2x3_f32;

impl VertexAttributeFormatIdentifier for Float2x3_f32 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2x3_f32;
}

pub struct Float2x3_i8_fixed;

impl VertexAttributeFormatIdentifier for Float2x3_i8_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2x3_i8_fixed;
}

pub struct Float2x3_i8_norm;

impl VertexAttributeFormatIdentifier for Float2x3_i8_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2x3_i8_norm;
}

pub struct Float2x3_i16_fixed;

impl VertexAttributeFormatIdentifier for Float2x3_i16_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2x3_i16_fixed;
}

pub struct Float2x3_i16_norm;

impl VertexAttributeFormatIdentifier for Float2x3_i16_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2x3_i16_norm;
}

pub struct Float2x3_u8_fixed;

impl VertexAttributeFormatIdentifier for Float2x3_u8_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2x3_u8_fixed;
}

pub struct Float2x3_u8_norm;

impl VertexAttributeFormatIdentifier for Float2x3_u8_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2x3_u8_norm;
}

pub struct Float2x3_u16_fixed;

impl VertexAttributeFormatIdentifier for Float2x3_u16_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2x3_u16_fixed;
}

pub struct Float2x3_u16_norm;

impl VertexAttributeFormatIdentifier for Float2x3_u16_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2x3_u16_norm;
}

pub struct Float2x4_f32;

impl VertexAttributeFormatIdentifier for Float2x4_f32 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2x4_f32;
}

pub struct Float2x4_i8_fixed;

impl VertexAttributeFormatIdentifier for Float2x4_i8_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2x4_i8_fixed;
}

pub struct Float2x4_i8_norm;

impl VertexAttributeFormatIdentifier for Float2x4_i8_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2x4_i8_norm;
}

pub struct Float2x4_i16_fixed;

impl VertexAttributeFormatIdentifier for Float2x4_i16_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2x4_i16_fixed;
}

pub struct Float2x4_i16_norm;

impl VertexAttributeFormatIdentifier for Float2x4_i16_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2x4_i16_norm;
}

pub struct Float2x4_u8_fixed;

impl VertexAttributeFormatIdentifier for Float2x4_u8_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2x4_u8_fixed;
}

pub struct Float2x4_u8_norm;

impl VertexAttributeFormatIdentifier for Float2x4_u8_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2x4_u8_norm;
}

pub struct Float2x4_u16_fixed;

impl VertexAttributeFormatIdentifier for Float2x4_u16_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2x4_u16_fixed;
}

pub struct Float2x4_u16_norm;

impl VertexAttributeFormatIdentifier for Float2x4_u16_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float2x4_u16_norm;
}

pub struct Float3x2_f32;

impl VertexAttributeFormatIdentifier for Float3x2_f32 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3x2_f32;
}

pub struct Float3x2_i8_fixed;

impl VertexAttributeFormatIdentifier for Float3x2_i8_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3x2_i8_fixed;
}

pub struct Float3x2_i8_norm;

impl VertexAttributeFormatIdentifier for Float3x2_i8_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3x2_i8_norm;
}

pub struct Float3x2_i16_fixed;

impl VertexAttributeFormatIdentifier for Float3x2_i16_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3x2_i16_fixed;
}

pub struct Float3x2_i16_norm;

impl VertexAttributeFormatIdentifier for Float3x2_i16_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3x2_i16_norm;
}

pub struct Float3x2_u8_fixed;

impl VertexAttributeFormatIdentifier for Float3x2_u8_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3x2_u8_fixed;
}

pub struct Float3x2_u8_norm;

impl VertexAttributeFormatIdentifier for Float3x2_u8_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3x2_u8_norm;
}

pub struct Float3x2_u16_fixed;

impl VertexAttributeFormatIdentifier for Float3x2_u16_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3x2_u16_fixed;
}

pub struct Float3x2_u16_norm;

impl VertexAttributeFormatIdentifier for Float3x2_u16_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3x2_u16_norm;
}

pub struct Float3x3_f32;

impl VertexAttributeFormatIdentifier for Float3x3_f32 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3x3_f32;
}

pub struct Float3x3_i8_fixed;

impl VertexAttributeFormatIdentifier for Float3x3_i8_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3x3_i8_fixed;
}

pub struct Float3x3_i8_norm;

impl VertexAttributeFormatIdentifier for Float3x3_i8_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3x3_i8_norm;
}

pub struct Float3x3_i16_fixed;

impl VertexAttributeFormatIdentifier for Float3x3_i16_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3x3_i16_fixed;
}

pub struct Float3x3_i16_norm;

impl VertexAttributeFormatIdentifier for Float3x3_i16_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3x3_i16_norm;
}

pub struct Float3x3_u8_fixed;

impl VertexAttributeFormatIdentifier for Float3x3_u8_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3x3_u8_fixed;
}

pub struct Float3x3_u8_norm;

impl VertexAttributeFormatIdentifier for Float3x3_u8_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3x3_u8_norm;
}

pub struct Float3x3_u16_fixed;

impl VertexAttributeFormatIdentifier for Float3x3_u16_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3x3_u16_fixed;
}

pub struct Float3x3_u16_norm;

impl VertexAttributeFormatIdentifier for Float3x3_u16_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3x3_u16_norm;
}

pub struct Float3x4_f32;

impl VertexAttributeFormatIdentifier for Float3x4_f32 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3x4_f32;
}

pub struct Float3x4_i8_fixed;

impl VertexAttributeFormatIdentifier for Float3x4_i8_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3x4_i8_fixed;
}

pub struct Float3x4_i8_norm;

impl VertexAttributeFormatIdentifier for Float3x4_i8_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3x4_i8_norm;
}

pub struct Float3x4_i16_fixed;

impl VertexAttributeFormatIdentifier for Float3x4_i16_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3x4_i16_fixed;
}

pub struct Float3x4_i16_norm;

impl VertexAttributeFormatIdentifier for Float3x4_i16_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3x4_i16_norm;
}

pub struct Float3x4_u8_fixed;

impl VertexAttributeFormatIdentifier for Float3x4_u8_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3x4_u8_fixed;
}

pub struct Float3x4_u8_norm;

impl VertexAttributeFormatIdentifier for Float3x4_u8_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3x4_u8_norm;
}

pub struct Float3x4_u16_fixed;

impl VertexAttributeFormatIdentifier for Float3x4_u16_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3x4_u16_fixed;
}

pub struct Float3x4_u16_norm;

impl VertexAttributeFormatIdentifier for Float3x4_u16_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float3x4_u16_norm;
}

pub struct Float4x2_f32;

impl VertexAttributeFormatIdentifier for Float4x2_f32 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4x2_f32;
}

pub struct Float4x2_i8_fixed;

impl VertexAttributeFormatIdentifier for Float4x2_i8_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4x2_i8_fixed;
}

pub struct Float4x2_i8_norm;

impl VertexAttributeFormatIdentifier for Float4x2_i8_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4x2_i8_norm;
}

pub struct Float4x2_i16_fixed;

impl VertexAttributeFormatIdentifier for Float4x2_i16_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4x2_i16_fixed;
}

pub struct Float4x2_i16_norm;

impl VertexAttributeFormatIdentifier for Float4x2_i16_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4x2_i16_norm;
}

pub struct Float4x2_u8_fixed;

impl VertexAttributeFormatIdentifier for Float4x2_u8_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4x2_u8_fixed;
}

pub struct Float4x2_u8_norm;

impl VertexAttributeFormatIdentifier for Float4x2_u8_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4x2_u8_norm;
}

pub struct Float4x2_u16_fixed;

impl VertexAttributeFormatIdentifier for Float4x2_u16_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4x2_u16_fixed;
}

pub struct Float4x2_u16_norm;

impl VertexAttributeFormatIdentifier for Float4x2_u16_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4x2_u16_norm;
}

pub struct Float4x3_f32;

impl VertexAttributeFormatIdentifier for Float4x3_f32 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4x3_f32;
}

pub struct Float4x3_i8_fixed;

impl VertexAttributeFormatIdentifier for Float4x3_i8_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4x3_i8_fixed;
}

pub struct Float4x3_i8_norm;

impl VertexAttributeFormatIdentifier for Float4x3_i8_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4x3_i8_norm;
}

pub struct Float4x3_i16_fixed;

impl VertexAttributeFormatIdentifier for Float4x3_i16_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4x3_i16_fixed;
}

pub struct Float4x3_i16_norm;

impl VertexAttributeFormatIdentifier for Float4x3_i16_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4x3_i16_norm;
}

pub struct Float4x3_u8_fixed;

impl VertexAttributeFormatIdentifier for Float4x3_u8_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4x3_u8_fixed;
}

pub struct Float4x3_u8_norm;

impl VertexAttributeFormatIdentifier for Float4x3_u8_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4x3_u8_norm;
}

pub struct Float4x3_u16_fixed;

impl VertexAttributeFormatIdentifier for Float4x3_u16_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4x3_u16_fixed;
}

pub struct Float4x3_u16_norm;

impl VertexAttributeFormatIdentifier for Float4x3_u16_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4x3_u16_norm;
}

pub struct Float4x4_f32;

impl VertexAttributeFormatIdentifier for Float4x4_f32 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4x4_f32;
}

pub struct Float4x4_i8_fixed;

impl VertexAttributeFormatIdentifier for Float4x4_i8_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4x4_i8_fixed;
}

pub struct Float4x4_i8_norm;

impl VertexAttributeFormatIdentifier for Float4x4_i8_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4x4_i8_norm;
}

pub struct Float4x4_i16_fixed;

impl VertexAttributeFormatIdentifier for Float4x4_i16_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4x4_i16_fixed;
}

pub struct Float4x4_i16_norm;

impl VertexAttributeFormatIdentifier for Float4x4_i16_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4x4_i16_norm;
}

pub struct Float4x4_u8_fixed;

impl VertexAttributeFormatIdentifier for Float4x4_u8_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4x4_u8_fixed;
}

pub struct Float4x4_u8_norm;

impl VertexAttributeFormatIdentifier for Float4x4_u8_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4x4_u8_norm;
}

pub struct Float4x4_u16_fixed;

impl VertexAttributeFormatIdentifier for Float4x4_u16_fixed {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4x4_u16_fixed;
}

pub struct Float4x4_u16_norm;

impl VertexAttributeFormatIdentifier for Float4x4_u16_norm {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Float4x4_u16_norm;
}

pub struct Integer_i8;

impl VertexAttributeFormatIdentifier for Integer_i8 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Integer_i8;
}

pub struct Integer_u8;

impl VertexAttributeFormatIdentifier for Integer_u8 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Integer_u8;
}

pub struct Integer_i16;

impl VertexAttributeFormatIdentifier for Integer_i16 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Integer_i16;
}

pub struct Integer_u16;

impl VertexAttributeFormatIdentifier for Integer_u16 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Integer_u16;
}

pub struct Integer_i32;

impl VertexAttributeFormatIdentifier for Integer_i32 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Integer_i32;
}

pub struct Integer_u32;

impl VertexAttributeFormatIdentifier for Integer_u32 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Integer_u32;
}

pub struct Integer2_i8;

impl VertexAttributeFormatIdentifier for Integer2_i8 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Integer2_i8;
}

pub struct Integer2_u8;

impl VertexAttributeFormatIdentifier for Integer2_u8 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Integer2_u8;
}

pub struct Integer2_i16;

impl VertexAttributeFormatIdentifier for Integer2_i16 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Integer2_i16;
}

pub struct Integer2_u16;

impl VertexAttributeFormatIdentifier for Integer2_u16 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Integer2_u16;
}

pub struct Integer2_i32;

impl VertexAttributeFormatIdentifier for Integer2_i32 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Integer2_i32;
}

pub struct Integer2_u32;

impl VertexAttributeFormatIdentifier for Integer2_u32 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Integer2_u32;
}

pub struct Integer3_i8;

impl VertexAttributeFormatIdentifier for Integer3_i8 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Integer3_i8;
}

pub struct Integer3_u8;

impl VertexAttributeFormatIdentifier for Integer3_u8 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Integer3_u8;
}

pub struct Integer3_i16;

impl VertexAttributeFormatIdentifier for Integer3_i16 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Integer3_i16;
}

pub struct Integer3_u16;

impl VertexAttributeFormatIdentifier for Integer3_u16 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Integer3_u16;
}

pub struct Integer3_i32;

impl VertexAttributeFormatIdentifier for Integer3_i32 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Integer3_i32;
}

pub struct Integer3_u32;

impl VertexAttributeFormatIdentifier for Integer3_u32 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Integer3_u32;
}

pub struct Integer4_i8;

impl VertexAttributeFormatIdentifier for Integer4_i8 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Integer4_i8;
}

pub struct Integer4_u8;

impl VertexAttributeFormatIdentifier for Integer4_u8 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Integer4_u8;
}

pub struct Integer4_i16;

impl VertexAttributeFormatIdentifier for Integer4_i16 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Integer4_i16;
}

pub struct Integer4_u16;

impl VertexAttributeFormatIdentifier for Integer4_u16 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Integer4_u16;
}

pub struct Integer4_i32;

impl VertexAttributeFormatIdentifier for Integer4_i32 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Integer4_i32;
}

pub struct Integer4_u32;

impl VertexAttributeFormatIdentifier for Integer4_u32 {
    const FORMAT: VertexAttributeFormat = VertexAttributeFormat::Integer4_u32;
}

/// Trait implemented for types that are memory compatible with the attribute format associated with
/// an [AttributeFormatIdentifier].
///
/// If a type implemented `FormatCompatible<F>`, where `F` is an [AttributeFormatIdentifier], then
/// that type can be used as the field type for a [Vertex] field that is marked as an attribute with
/// format `F`.
///
/// See also [Vertex].
///
/// # Unsafe
///
/// Only safe to implement for a type if the memory for any value of that type can be cast to an
/// attribute value in the format associated with the [AttributeFormatIdentifier].
pub unsafe trait VertexAttributeFormatCompatible<F>
where
    F: VertexAttributeFormatIdentifier,
{
}

unsafe impl VertexAttributeFormatCompatible<Float_f32> for f32 {}
unsafe impl VertexAttributeFormatCompatible<Float_i8_fixed> for i8 {}
unsafe impl VertexAttributeFormatCompatible<Float_i8_norm> for i8 {}
unsafe impl VertexAttributeFormatCompatible<Float_u8_fixed> for u8 {}
unsafe impl VertexAttributeFormatCompatible<Float_u8_norm> for u8 {}
unsafe impl VertexAttributeFormatCompatible<Float_i16_fixed> for i16 {}
unsafe impl VertexAttributeFormatCompatible<Float_i16_norm> for i16 {}
unsafe impl VertexAttributeFormatCompatible<Float_u16_fixed> for u16 {}
unsafe impl VertexAttributeFormatCompatible<Float_u16_norm> for u16 {}
unsafe impl VertexAttributeFormatCompatible<Float2_f32> for [f32; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2_i8_fixed> for [i8; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2_i8_norm> for [i8; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2_u8_fixed> for [u8; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2_u8_norm> for [u8; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2_i16_fixed> for [i16; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2_i16_norm> for [i16; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2_u16_fixed> for [u16; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2_u16_norm> for [u16; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float3_f32> for [f32; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3_i8_fixed> for [i8; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3_i8_norm> for [i8; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3_u8_fixed> for [u8; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3_u8_norm> for [u8; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3_i16_fixed> for [i16; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3_i16_norm> for [i16; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3_u16_fixed> for [u16; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3_u16_norm> for [u16; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float4_f32> for [f32; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4_i8_fixed> for [i8; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4_i8_norm> for [i8; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4_u8_fixed> for [u8; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4_u8_norm> for [u8; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4_i16_fixed> for [i16; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4_i16_norm> for [i16; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4_u16_fixed> for [u16; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4_u16_norm> for [u16; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float2x2_f32> for [[f32; 2]; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2x2_i8_fixed> for [[i8; 2]; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2x2_i8_norm> for [[i8; 2]; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2x2_u8_fixed> for [[u8; 2]; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2x2_u8_norm> for [[u8; 2]; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2x2_i16_fixed> for [[i16; 2]; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2x2_i16_norm> for [[i16; 2]; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2x2_u16_fixed> for [[u16; 2]; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2x2_u16_norm> for [[u16; 2]; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2x3_f32> for [[f32; 3]; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2x3_i8_fixed> for [[i8; 3]; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2x3_i8_norm> for [[i8; 3]; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2x3_u8_fixed> for [[u8; 3]; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2x3_u8_norm> for [[u8; 3]; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2x3_i16_fixed> for [[i16; 3]; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2x3_i16_norm> for [[i16; 3]; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2x3_u16_fixed> for [[u16; 3]; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2x3_u16_norm> for [[u16; 3]; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2x4_f32> for [[f32; 4]; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2x4_i8_fixed> for [[i8; 4]; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2x4_i8_norm> for [[i8; 4]; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2x4_u8_fixed> for [[u8; 4]; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2x4_u8_norm> for [[u8; 4]; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2x4_i16_fixed> for [[i16; 4]; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2x4_i16_norm> for [[i16; 4]; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2x4_u16_fixed> for [[u16; 4]; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float2x4_u16_norm> for [[u16; 4]; 2] {}
unsafe impl VertexAttributeFormatCompatible<Float3x2_f32> for [[f32; 2]; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3x2_i8_fixed> for [[i8; 2]; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3x2_i8_norm> for [[i8; 2]; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3x2_u8_fixed> for [[u8; 2]; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3x2_u8_norm> for [[u8; 2]; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3x2_i16_fixed> for [[i16; 2]; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3x2_i16_norm> for [[i16; 2]; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3x2_u16_fixed> for [[u16; 2]; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3x2_u16_norm> for [[u16; 2]; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3x3_f32> for [[f32; 3]; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3x3_i8_fixed> for [[i8; 3]; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3x3_i8_norm> for [[i8; 3]; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3x3_u8_fixed> for [[u8; 3]; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3x3_u8_norm> for [[u8; 3]; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3x3_i16_fixed> for [[i16; 3]; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3x3_i16_norm> for [[i16; 3]; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3x3_u16_fixed> for [[u16; 3]; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3x3_u16_norm> for [[u16; 3]; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3x4_f32> for [[f32; 4]; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3x4_i8_fixed> for [[i8; 4]; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3x4_i8_norm> for [[i8; 4]; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3x4_u8_fixed> for [[u8; 4]; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3x4_u8_norm> for [[u8; 4]; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3x4_i16_fixed> for [[i16; 4]; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3x4_i16_norm> for [[i16; 4]; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3x4_u16_fixed> for [[u16; 4]; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float3x4_u16_norm> for [[u16; 4]; 3] {}
unsafe impl VertexAttributeFormatCompatible<Float4x2_f32> for [[f32; 2]; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4x2_i8_fixed> for [[i8; 2]; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4x2_i8_norm> for [[i8; 2]; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4x2_u8_fixed> for [[u8; 2]; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4x2_u8_norm> for [[u8; 2]; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4x2_i16_fixed> for [[i16; 2]; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4x2_i16_norm> for [[i16; 2]; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4x2_u16_fixed> for [[u16; 2]; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4x2_u16_norm> for [[u16; 2]; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4x3_f32> for [[f32; 3]; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4x3_i8_fixed> for [[i8; 3]; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4x3_i8_norm> for [[i8; 3]; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4x3_u8_fixed> for [[u8; 3]; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4x3_u8_norm> for [[u8; 3]; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4x3_i16_fixed> for [[i16; 3]; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4x3_i16_norm> for [[i16; 3]; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4x3_u16_fixed> for [[u16; 3]; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4x3_u16_norm> for [[u16; 3]; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4x4_f32> for [[f32; 4]; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4x4_i8_fixed> for [[i8; 4]; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4x4_i8_norm> for [[i8; 4]; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4x4_u8_fixed> for [[u8; 4]; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4x4_u8_norm> for [[u8; 4]; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4x4_i16_fixed> for [[i16; 4]; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4x4_i16_norm> for [[i16; 4]; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4x4_u16_fixed> for [[u16; 4]; 4] {}
unsafe impl VertexAttributeFormatCompatible<Float4x4_u16_norm> for [[u16; 4]; 4] {}
unsafe impl VertexAttributeFormatCompatible<Integer_i8> for i8 {}
unsafe impl VertexAttributeFormatCompatible<Integer_i16> for i16 {}
unsafe impl VertexAttributeFormatCompatible<Integer_i32> for i32 {}
unsafe impl VertexAttributeFormatCompatible<Integer_u8> for u8 {}
unsafe impl VertexAttributeFormatCompatible<Integer_u16> for u16 {}
unsafe impl VertexAttributeFormatCompatible<Integer_u32> for u32 {}
unsafe impl VertexAttributeFormatCompatible<Integer2_i8> for [i8; 2] {}
unsafe impl VertexAttributeFormatCompatible<Integer2_i16> for [i16; 2] {}
unsafe impl VertexAttributeFormatCompatible<Integer2_i32> for [i32; 2] {}
unsafe impl VertexAttributeFormatCompatible<Integer2_u8> for [u8; 2] {}
unsafe impl VertexAttributeFormatCompatible<Integer2_u16> for [u16; 2] {}
unsafe impl VertexAttributeFormatCompatible<Integer2_u32> for [u32; 2] {}
unsafe impl VertexAttributeFormatCompatible<Integer3_i8> for [i8; 3] {}
unsafe impl VertexAttributeFormatCompatible<Integer3_i16> for [i16; 3] {}
unsafe impl VertexAttributeFormatCompatible<Integer3_i32> for [i32; 3] {}
unsafe impl VertexAttributeFormatCompatible<Integer3_u8> for [u8; 3] {}
unsafe impl VertexAttributeFormatCompatible<Integer3_u16> for [u16; 3] {}
unsafe impl VertexAttributeFormatCompatible<Integer3_u32> for [u32; 3] {}
unsafe impl VertexAttributeFormatCompatible<Integer4_i8> for [i8; 4] {}
unsafe impl VertexAttributeFormatCompatible<Integer4_i16> for [i16; 4] {}
unsafe impl VertexAttributeFormatCompatible<Integer4_i32> for [i32; 4] {}
unsafe impl VertexAttributeFormatCompatible<Integer4_u8> for [u8; 4] {}
unsafe impl VertexAttributeFormatCompatible<Integer4_u16> for [u16; 4] {}
unsafe impl VertexAttributeFormatCompatible<Integer4_u32> for [u32; 4] {}

/// Enumerates all available attribute memory formats.
#[derive(Clone, Copy, PartialEq, Hash, Debug)]
#[allow(non_camel_case_types)]
pub enum VertexAttributeFormat {
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

impl VertexAttributeFormat {
    /// Whether or not this [AttributeFormat] is compatible with an [AttributeSlotDescriptor] of
    /// the given [AttributeType].
    pub fn is_compatible(&self, attribute_type: VertexAttributeType) -> bool {
        match self {
            VertexAttributeFormat::Float_f32 => attribute_type == VertexAttributeType::Float,
            VertexAttributeFormat::Float_i8_fixed => attribute_type == VertexAttributeType::Float,
            VertexAttributeFormat::Float_i8_norm => attribute_type == VertexAttributeType::Float,
            VertexAttributeFormat::Float_i16_fixed => attribute_type == VertexAttributeType::Float,
            VertexAttributeFormat::Float_i16_norm => attribute_type == VertexAttributeType::Float,
            VertexAttributeFormat::Float_u8_fixed => attribute_type == VertexAttributeType::Float,
            VertexAttributeFormat::Float_u8_norm => attribute_type == VertexAttributeType::Float,
            VertexAttributeFormat::Float_u16_fixed => attribute_type == VertexAttributeType::Float,
            VertexAttributeFormat::Float_u16_norm => attribute_type == VertexAttributeType::Float,
            VertexAttributeFormat::Float2_f32 => {
                attribute_type == VertexAttributeType::FloatVector2
            }
            VertexAttributeFormat::Float2_i8_fixed => {
                attribute_type == VertexAttributeType::FloatVector2
            }
            VertexAttributeFormat::Float2_i8_norm => {
                attribute_type == VertexAttributeType::FloatVector2
            }
            VertexAttributeFormat::Float2_i16_fixed => {
                attribute_type == VertexAttributeType::FloatVector2
            }
            VertexAttributeFormat::Float2_i16_norm => {
                attribute_type == VertexAttributeType::FloatVector2
            }
            VertexAttributeFormat::Float2_u8_fixed => {
                attribute_type == VertexAttributeType::FloatVector2
            }
            VertexAttributeFormat::Float2_u8_norm => {
                attribute_type == VertexAttributeType::FloatVector2
            }
            VertexAttributeFormat::Float2_u16_fixed => {
                attribute_type == VertexAttributeType::FloatVector2
            }
            VertexAttributeFormat::Float2_u16_norm => {
                attribute_type == VertexAttributeType::FloatVector2
            }
            VertexAttributeFormat::Float3_f32 => {
                attribute_type == VertexAttributeType::FloatVector3
            }
            VertexAttributeFormat::Float3_i8_fixed => {
                attribute_type == VertexAttributeType::FloatVector3
            }
            VertexAttributeFormat::Float3_i8_norm => {
                attribute_type == VertexAttributeType::FloatVector3
            }
            VertexAttributeFormat::Float3_i16_fixed => {
                attribute_type == VertexAttributeType::FloatVector3
            }
            VertexAttributeFormat::Float3_i16_norm => {
                attribute_type == VertexAttributeType::FloatVector3
            }
            VertexAttributeFormat::Float3_u8_fixed => {
                attribute_type == VertexAttributeType::FloatVector3
            }
            VertexAttributeFormat::Float3_u8_norm => {
                attribute_type == VertexAttributeType::FloatVector3
            }
            VertexAttributeFormat::Float3_u16_fixed => {
                attribute_type == VertexAttributeType::FloatVector3
            }
            VertexAttributeFormat::Float3_u16_norm => {
                attribute_type == VertexAttributeType::FloatVector3
            }
            VertexAttributeFormat::Float4_f32 => {
                attribute_type == VertexAttributeType::FloatVector4
            }
            VertexAttributeFormat::Float4_i8_fixed => {
                attribute_type == VertexAttributeType::FloatVector4
            }
            VertexAttributeFormat::Float4_i8_norm => {
                attribute_type == VertexAttributeType::FloatVector4
            }
            VertexAttributeFormat::Float4_i16_fixed => {
                attribute_type == VertexAttributeType::FloatVector4
            }
            VertexAttributeFormat::Float4_i16_norm => {
                attribute_type == VertexAttributeType::FloatVector4
            }
            VertexAttributeFormat::Float4_u8_fixed => {
                attribute_type == VertexAttributeType::FloatVector4
            }
            VertexAttributeFormat::Float4_u8_norm => {
                attribute_type == VertexAttributeType::FloatVector4
            }
            VertexAttributeFormat::Float4_u16_fixed => {
                attribute_type == VertexAttributeType::FloatVector4
            }
            VertexAttributeFormat::Float4_u16_norm => {
                attribute_type == VertexAttributeType::FloatVector4
            }
            VertexAttributeFormat::Float2x2_f32 => {
                attribute_type == VertexAttributeType::FloatMatrix2x2
            }
            VertexAttributeFormat::Float2x2_i8_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix2x2
            }
            VertexAttributeFormat::Float2x2_i8_norm => {
                attribute_type == VertexAttributeType::FloatMatrix2x2
            }
            VertexAttributeFormat::Float2x2_i16_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix2x2
            }
            VertexAttributeFormat::Float2x2_i16_norm => {
                attribute_type == VertexAttributeType::FloatMatrix2x2
            }
            VertexAttributeFormat::Float2x2_u8_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix2x2
            }
            VertexAttributeFormat::Float2x2_u8_norm => {
                attribute_type == VertexAttributeType::FloatMatrix2x2
            }
            VertexAttributeFormat::Float2x2_u16_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix2x2
            }
            VertexAttributeFormat::Float2x2_u16_norm => {
                attribute_type == VertexAttributeType::FloatMatrix2x2
            }
            VertexAttributeFormat::Float2x3_f32 => {
                attribute_type == VertexAttributeType::FloatMatrix2x3
            }
            VertexAttributeFormat::Float2x3_i8_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix2x3
            }
            VertexAttributeFormat::Float2x3_i8_norm => {
                attribute_type == VertexAttributeType::FloatMatrix2x3
            }
            VertexAttributeFormat::Float2x3_i16_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix2x3
            }
            VertexAttributeFormat::Float2x3_i16_norm => {
                attribute_type == VertexAttributeType::FloatMatrix2x3
            }
            VertexAttributeFormat::Float2x3_u8_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix2x3
            }
            VertexAttributeFormat::Float2x3_u8_norm => {
                attribute_type == VertexAttributeType::FloatMatrix2x3
            }
            VertexAttributeFormat::Float2x3_u16_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix2x3
            }
            VertexAttributeFormat::Float2x3_u16_norm => {
                attribute_type == VertexAttributeType::FloatMatrix2x3
            }
            VertexAttributeFormat::Float2x4_f32 => {
                attribute_type == VertexAttributeType::FloatMatrix2x4
            }
            VertexAttributeFormat::Float2x4_i8_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix2x4
            }
            VertexAttributeFormat::Float2x4_i8_norm => {
                attribute_type == VertexAttributeType::FloatMatrix2x4
            }
            VertexAttributeFormat::Float2x4_i16_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix2x4
            }
            VertexAttributeFormat::Float2x4_i16_norm => {
                attribute_type == VertexAttributeType::FloatMatrix2x4
            }
            VertexAttributeFormat::Float2x4_u8_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix2x4
            }
            VertexAttributeFormat::Float2x4_u8_norm => {
                attribute_type == VertexAttributeType::FloatMatrix2x4
            }
            VertexAttributeFormat::Float2x4_u16_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix2x4
            }
            VertexAttributeFormat::Float2x4_u16_norm => {
                attribute_type == VertexAttributeType::FloatMatrix2x4
            }
            VertexAttributeFormat::Float3x2_f32 => {
                attribute_type == VertexAttributeType::FloatMatrix3x2
            }
            VertexAttributeFormat::Float3x2_i8_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix3x2
            }
            VertexAttributeFormat::Float3x2_i8_norm => {
                attribute_type == VertexAttributeType::FloatMatrix3x2
            }
            VertexAttributeFormat::Float3x2_i16_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix3x2
            }
            VertexAttributeFormat::Float3x2_i16_norm => {
                attribute_type == VertexAttributeType::FloatMatrix3x2
            }
            VertexAttributeFormat::Float3x2_u8_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix3x2
            }
            VertexAttributeFormat::Float3x2_u8_norm => {
                attribute_type == VertexAttributeType::FloatMatrix3x2
            }
            VertexAttributeFormat::Float3x2_u16_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix3x2
            }
            VertexAttributeFormat::Float3x2_u16_norm => {
                attribute_type == VertexAttributeType::FloatMatrix3x2
            }
            VertexAttributeFormat::Float3x3_f32 => {
                attribute_type == VertexAttributeType::FloatMatrix3x3
            }
            VertexAttributeFormat::Float3x3_i8_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix3x3
            }
            VertexAttributeFormat::Float3x3_i8_norm => {
                attribute_type == VertexAttributeType::FloatMatrix3x3
            }
            VertexAttributeFormat::Float3x3_i16_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix3x3
            }
            VertexAttributeFormat::Float3x3_i16_norm => {
                attribute_type == VertexAttributeType::FloatMatrix3x3
            }
            VertexAttributeFormat::Float3x3_u8_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix3x3
            }
            VertexAttributeFormat::Float3x3_u8_norm => {
                attribute_type == VertexAttributeType::FloatMatrix3x3
            }
            VertexAttributeFormat::Float3x3_u16_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix3x3
            }
            VertexAttributeFormat::Float3x3_u16_norm => {
                attribute_type == VertexAttributeType::FloatMatrix3x3
            }
            VertexAttributeFormat::Float3x4_f32 => {
                attribute_type == VertexAttributeType::FloatMatrix3x4
            }
            VertexAttributeFormat::Float3x4_i8_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix3x4
            }
            VertexAttributeFormat::Float3x4_i8_norm => {
                attribute_type == VertexAttributeType::FloatMatrix3x4
            }
            VertexAttributeFormat::Float3x4_i16_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix3x4
            }
            VertexAttributeFormat::Float3x4_i16_norm => {
                attribute_type == VertexAttributeType::FloatMatrix3x4
            }
            VertexAttributeFormat::Float3x4_u8_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix3x4
            }
            VertexAttributeFormat::Float3x4_u8_norm => {
                attribute_type == VertexAttributeType::FloatMatrix3x4
            }
            VertexAttributeFormat::Float3x4_u16_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix3x4
            }
            VertexAttributeFormat::Float3x4_u16_norm => {
                attribute_type == VertexAttributeType::FloatMatrix3x4
            }
            VertexAttributeFormat::Float4x2_f32 => {
                attribute_type == VertexAttributeType::FloatMatrix4x2
            }
            VertexAttributeFormat::Float4x2_i8_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix4x2
            }
            VertexAttributeFormat::Float4x2_i8_norm => {
                attribute_type == VertexAttributeType::FloatMatrix4x2
            }
            VertexAttributeFormat::Float4x2_i16_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix4x2
            }
            VertexAttributeFormat::Float4x2_i16_norm => {
                attribute_type == VertexAttributeType::FloatMatrix4x2
            }
            VertexAttributeFormat::Float4x2_u8_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix4x2
            }
            VertexAttributeFormat::Float4x2_u8_norm => {
                attribute_type == VertexAttributeType::FloatMatrix4x2
            }
            VertexAttributeFormat::Float4x2_u16_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix4x2
            }
            VertexAttributeFormat::Float4x2_u16_norm => {
                attribute_type == VertexAttributeType::FloatMatrix4x2
            }
            VertexAttributeFormat::Float4x3_f32 => {
                attribute_type == VertexAttributeType::FloatMatrix4x3
            }
            VertexAttributeFormat::Float4x3_i8_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix4x3
            }
            VertexAttributeFormat::Float4x3_i8_norm => {
                attribute_type == VertexAttributeType::FloatMatrix4x3
            }
            VertexAttributeFormat::Float4x3_i16_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix4x3
            }
            VertexAttributeFormat::Float4x3_i16_norm => {
                attribute_type == VertexAttributeType::FloatMatrix4x3
            }
            VertexAttributeFormat::Float4x3_u8_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix4x3
            }
            VertexAttributeFormat::Float4x3_u8_norm => {
                attribute_type == VertexAttributeType::FloatMatrix4x3
            }
            VertexAttributeFormat::Float4x3_u16_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix4x3
            }
            VertexAttributeFormat::Float4x3_u16_norm => {
                attribute_type == VertexAttributeType::FloatMatrix4x3
            }
            VertexAttributeFormat::Float4x4_f32 => {
                attribute_type == VertexAttributeType::FloatMatrix4x4
            }
            VertexAttributeFormat::Float4x4_i8_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix4x4
            }
            VertexAttributeFormat::Float4x4_i8_norm => {
                attribute_type == VertexAttributeType::FloatMatrix4x4
            }
            VertexAttributeFormat::Float4x4_i16_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix4x4
            }
            VertexAttributeFormat::Float4x4_i16_norm => {
                attribute_type == VertexAttributeType::FloatMatrix4x4
            }
            VertexAttributeFormat::Float4x4_u8_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix4x4
            }
            VertexAttributeFormat::Float4x4_u8_norm => {
                attribute_type == VertexAttributeType::FloatMatrix4x4
            }
            VertexAttributeFormat::Float4x4_u16_fixed => {
                attribute_type == VertexAttributeType::FloatMatrix4x4
            }
            VertexAttributeFormat::Float4x4_u16_norm => {
                attribute_type == VertexAttributeType::FloatMatrix4x4
            }
            VertexAttributeFormat::Integer_i8 => attribute_type == VertexAttributeType::Integer,
            VertexAttributeFormat::Integer_u8 => {
                attribute_type == VertexAttributeType::UnsignedInteger
            }
            VertexAttributeFormat::Integer_i16 => attribute_type == VertexAttributeType::Integer,
            VertexAttributeFormat::Integer_u16 => {
                attribute_type == VertexAttributeType::UnsignedInteger
            }
            VertexAttributeFormat::Integer_i32 => attribute_type == VertexAttributeType::Integer,
            VertexAttributeFormat::Integer_u32 => {
                attribute_type == VertexAttributeType::UnsignedInteger
            }
            VertexAttributeFormat::Integer2_i8 => {
                attribute_type == VertexAttributeType::IntegerVector2
            }
            VertexAttributeFormat::Integer2_u8 => {
                attribute_type == VertexAttributeType::UnsignedIntegerVector2
            }
            VertexAttributeFormat::Integer2_i16 => {
                attribute_type == VertexAttributeType::IntegerVector2
            }
            VertexAttributeFormat::Integer2_u16 => {
                attribute_type == VertexAttributeType::UnsignedIntegerVector2
            }
            VertexAttributeFormat::Integer2_i32 => {
                attribute_type == VertexAttributeType::IntegerVector2
            }
            VertexAttributeFormat::Integer2_u32 => {
                attribute_type == VertexAttributeType::UnsignedIntegerVector2
            }
            VertexAttributeFormat::Integer3_i8 => {
                attribute_type == VertexAttributeType::IntegerVector3
            }
            VertexAttributeFormat::Integer3_u8 => {
                attribute_type == VertexAttributeType::UnsignedIntegerVector3
            }
            VertexAttributeFormat::Integer3_i16 => {
                attribute_type == VertexAttributeType::IntegerVector3
            }
            VertexAttributeFormat::Integer3_u16 => {
                attribute_type == VertexAttributeType::UnsignedIntegerVector3
            }
            VertexAttributeFormat::Integer3_i32 => {
                attribute_type == VertexAttributeType::IntegerVector3
            }
            VertexAttributeFormat::Integer3_u32 => {
                attribute_type == VertexAttributeType::UnsignedIntegerVector3
            }
            VertexAttributeFormat::Integer4_i8 => {
                attribute_type == VertexAttributeType::IntegerVector4
            }
            VertexAttributeFormat::Integer4_u8 => {
                attribute_type == VertexAttributeType::UnsignedIntegerVector4
            }
            VertexAttributeFormat::Integer4_i16 => {
                attribute_type == VertexAttributeType::IntegerVector4
            }
            VertexAttributeFormat::Integer4_u16 => {
                attribute_type == VertexAttributeType::UnsignedIntegerVector4
            }
            VertexAttributeFormat::Integer4_i32 => {
                attribute_type == VertexAttributeType::IntegerVector4
            }
            VertexAttributeFormat::Integer4_u32 => {
                attribute_type == VertexAttributeType::UnsignedIntegerVector4
            }
        }
    }

    pub fn size_in_bytes(&self) -> u8 {
        match self {
            VertexAttributeFormat::Float_f32 => 4,
            VertexAttributeFormat::Float_i8_fixed => 1,
            VertexAttributeFormat::Float_i8_norm => 1,
            VertexAttributeFormat::Float_i16_fixed => 2,
            VertexAttributeFormat::Float_i16_norm => 2,
            VertexAttributeFormat::Float_u8_fixed => 1,
            VertexAttributeFormat::Float_u8_norm => 1,
            VertexAttributeFormat::Float_u16_fixed => 2,
            VertexAttributeFormat::Float_u16_norm => 2,
            VertexAttributeFormat::Float2_f32 => 8,
            VertexAttributeFormat::Float2_i8_fixed => 2,
            VertexAttributeFormat::Float2_i8_norm => 2,
            VertexAttributeFormat::Float2_i16_fixed => 4,
            VertexAttributeFormat::Float2_i16_norm => 4,
            VertexAttributeFormat::Float2_u8_fixed => 2,
            VertexAttributeFormat::Float2_u8_norm => 2,
            VertexAttributeFormat::Float2_u16_fixed => 4,
            VertexAttributeFormat::Float2_u16_norm => 4,
            VertexAttributeFormat::Float3_f32 => 12,
            VertexAttributeFormat::Float3_i8_fixed => 3,
            VertexAttributeFormat::Float3_i8_norm => 3,
            VertexAttributeFormat::Float3_i16_fixed => 6,
            VertexAttributeFormat::Float3_i16_norm => 6,
            VertexAttributeFormat::Float3_u8_fixed => 3,
            VertexAttributeFormat::Float3_u8_norm => 3,
            VertexAttributeFormat::Float3_u16_fixed => 6,
            VertexAttributeFormat::Float3_u16_norm => 6,
            VertexAttributeFormat::Float4_f32 => 16,
            VertexAttributeFormat::Float4_i8_fixed => 4,
            VertexAttributeFormat::Float4_i8_norm => 4,
            VertexAttributeFormat::Float4_i16_fixed => 8,
            VertexAttributeFormat::Float4_i16_norm => 8,
            VertexAttributeFormat::Float4_u8_fixed => 4,
            VertexAttributeFormat::Float4_u8_norm => 4,
            VertexAttributeFormat::Float4_u16_fixed => 8,
            VertexAttributeFormat::Float4_u16_norm => 8,
            VertexAttributeFormat::Float2x2_f32 => 16,
            VertexAttributeFormat::Float2x2_i8_fixed => 4,
            VertexAttributeFormat::Float2x2_i8_norm => 4,
            VertexAttributeFormat::Float2x2_i16_fixed => 8,
            VertexAttributeFormat::Float2x2_i16_norm => 8,
            VertexAttributeFormat::Float2x2_u8_fixed => 4,
            VertexAttributeFormat::Float2x2_u8_norm => 4,
            VertexAttributeFormat::Float2x2_u16_fixed => 8,
            VertexAttributeFormat::Float2x2_u16_norm => 8,
            VertexAttributeFormat::Float2x3_f32 => 24,
            VertexAttributeFormat::Float2x3_i8_fixed => 6,
            VertexAttributeFormat::Float2x3_i8_norm => 6,
            VertexAttributeFormat::Float2x3_i16_fixed => 12,
            VertexAttributeFormat::Float2x3_i16_norm => 12,
            VertexAttributeFormat::Float2x3_u8_fixed => 6,
            VertexAttributeFormat::Float2x3_u8_norm => 6,
            VertexAttributeFormat::Float2x3_u16_fixed => 12,
            VertexAttributeFormat::Float2x3_u16_norm => 12,
            VertexAttributeFormat::Float2x4_f32 => 32,
            VertexAttributeFormat::Float2x4_i8_fixed => 8,
            VertexAttributeFormat::Float2x4_i8_norm => 8,
            VertexAttributeFormat::Float2x4_i16_fixed => 16,
            VertexAttributeFormat::Float2x4_i16_norm => 16,
            VertexAttributeFormat::Float2x4_u8_fixed => 8,
            VertexAttributeFormat::Float2x4_u8_norm => 8,
            VertexAttributeFormat::Float2x4_u16_fixed => 16,
            VertexAttributeFormat::Float2x4_u16_norm => 16,
            VertexAttributeFormat::Float3x2_f32 => 24,
            VertexAttributeFormat::Float3x2_i8_fixed => 6,
            VertexAttributeFormat::Float3x2_i8_norm => 6,
            VertexAttributeFormat::Float3x2_i16_fixed => 12,
            VertexAttributeFormat::Float3x2_i16_norm => 12,
            VertexAttributeFormat::Float3x2_u8_fixed => 6,
            VertexAttributeFormat::Float3x2_u8_norm => 6,
            VertexAttributeFormat::Float3x2_u16_fixed => 12,
            VertexAttributeFormat::Float3x2_u16_norm => 12,
            VertexAttributeFormat::Float3x3_f32 => 36,
            VertexAttributeFormat::Float3x3_i8_fixed => 9,
            VertexAttributeFormat::Float3x3_i8_norm => 9,
            VertexAttributeFormat::Float3x3_i16_fixed => 18,
            VertexAttributeFormat::Float3x3_i16_norm => 18,
            VertexAttributeFormat::Float3x3_u8_fixed => 9,
            VertexAttributeFormat::Float3x3_u8_norm => 9,
            VertexAttributeFormat::Float3x3_u16_fixed => 18,
            VertexAttributeFormat::Float3x3_u16_norm => 18,
            VertexAttributeFormat::Float3x4_f32 => 48,
            VertexAttributeFormat::Float3x4_i8_fixed => 12,
            VertexAttributeFormat::Float3x4_i8_norm => 12,
            VertexAttributeFormat::Float3x4_i16_fixed => 24,
            VertexAttributeFormat::Float3x4_i16_norm => 24,
            VertexAttributeFormat::Float3x4_u8_fixed => 12,
            VertexAttributeFormat::Float3x4_u8_norm => 12,
            VertexAttributeFormat::Float3x4_u16_fixed => 24,
            VertexAttributeFormat::Float3x4_u16_norm => 24,
            VertexAttributeFormat::Float4x2_f32 => 32,
            VertexAttributeFormat::Float4x2_i8_fixed => 8,
            VertexAttributeFormat::Float4x2_i8_norm => 8,
            VertexAttributeFormat::Float4x2_i16_fixed => 16,
            VertexAttributeFormat::Float4x2_i16_norm => 16,
            VertexAttributeFormat::Float4x2_u8_fixed => 8,
            VertexAttributeFormat::Float4x2_u8_norm => 8,
            VertexAttributeFormat::Float4x2_u16_fixed => 16,
            VertexAttributeFormat::Float4x2_u16_norm => 16,
            VertexAttributeFormat::Float4x3_f32 => 48,
            VertexAttributeFormat::Float4x3_i8_fixed => 12,
            VertexAttributeFormat::Float4x3_i8_norm => 12,
            VertexAttributeFormat::Float4x3_i16_fixed => 24,
            VertexAttributeFormat::Float4x3_i16_norm => 24,
            VertexAttributeFormat::Float4x3_u8_fixed => 12,
            VertexAttributeFormat::Float4x3_u8_norm => 12,
            VertexAttributeFormat::Float4x3_u16_fixed => 24,
            VertexAttributeFormat::Float4x3_u16_norm => 24,
            VertexAttributeFormat::Float4x4_f32 => 64,
            VertexAttributeFormat::Float4x4_i8_fixed => 16,
            VertexAttributeFormat::Float4x4_i8_norm => 16,
            VertexAttributeFormat::Float4x4_i16_fixed => 32,
            VertexAttributeFormat::Float4x4_i16_norm => 32,
            VertexAttributeFormat::Float4x4_u8_fixed => 16,
            VertexAttributeFormat::Float4x4_u8_norm => 16,
            VertexAttributeFormat::Float4x4_u16_fixed => 32,
            VertexAttributeFormat::Float4x4_u16_norm => 32,
            VertexAttributeFormat::Integer_i8 => 1,
            VertexAttributeFormat::Integer_u8 => 1,
            VertexAttributeFormat::Integer_i16 => 2,
            VertexAttributeFormat::Integer_u16 => 2,
            VertexAttributeFormat::Integer_i32 => 4,
            VertexAttributeFormat::Integer_u32 => 4,
            VertexAttributeFormat::Integer2_i8 => 2,
            VertexAttributeFormat::Integer2_u8 => 2,
            VertexAttributeFormat::Integer2_i16 => 4,
            VertexAttributeFormat::Integer2_u16 => 4,
            VertexAttributeFormat::Integer2_i32 => 8,
            VertexAttributeFormat::Integer2_u32 => 8,
            VertexAttributeFormat::Integer3_i8 => 3,
            VertexAttributeFormat::Integer3_u8 => 3,
            VertexAttributeFormat::Integer3_i16 => 6,
            VertexAttributeFormat::Integer3_u16 => 6,
            VertexAttributeFormat::Integer3_i32 => 12,
            VertexAttributeFormat::Integer3_u32 => 12,
            VertexAttributeFormat::Integer4_i8 => 4,
            VertexAttributeFormat::Integer4_u8 => 4,
            VertexAttributeFormat::Integer4_i16 => 8,
            VertexAttributeFormat::Integer4_u16 => 8,
            VertexAttributeFormat::Integer4_i32 => 16,
            VertexAttributeFormat::Integer4_u32 => 16,
        }
    }
}
