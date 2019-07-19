#![allow(non_camel_case_types)]

use crate::pipeline::graphics::AttributeType;

/// Trait implemented by attribute format identifiers.
///
/// Helper trait which, in conjunction with [FormatCompatible], allows the derive macro for the
/// [Vertex] trait to verify at compile time that an attribute field type is compatible with the
/// specified attribute format.
pub trait AttributeFormatIdentifier {
    /// The [AttributeFormat] associated with this [AttributeFormatIdentifier].
    const FORMAT: AttributeFormat;
}

pub struct Float_f32;

impl AttributeFormatIdentifier for Float_f32 {
    const FORMAT: AttributeFormat = AttributeFormat::Float_f32;
}

pub struct Float_i8_fixed;

impl AttributeFormatIdentifier for Float_i8_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float_i8_fixed;
}

pub struct Float_i8_norm;

impl AttributeFormatIdentifier for Float_i8_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float_i8_norm;
}

pub struct Float_i16_fixed;

impl AttributeFormatIdentifier for Float_i16_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float_i16_fixed;
}

pub struct Float_i16_norm;

impl AttributeFormatIdentifier for Float_i16_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float_i16_norm;
}

pub struct Float_u8_fixed;

impl AttributeFormatIdentifier for Float_u8_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float_u8_fixed;
}

pub struct Float_u8_norm;

impl AttributeFormatIdentifier for Float_u8_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float_u8_norm;
}

pub struct Float_u16_fixed;

impl AttributeFormatIdentifier for Float_u16_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float_u16_fixed;
}

pub struct Float_u16_norm;

impl AttributeFormatIdentifier for Float_u16_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float_u16_norm;
}

pub struct Float2_f32;

impl AttributeFormatIdentifier for Float2_f32 {
    const FORMAT: AttributeFormat = AttributeFormat::Float2_f32;
}

pub struct Float2_i8_fixed;

impl AttributeFormatIdentifier for Float2_i8_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float2_i8_fixed;
}

pub struct Float2_i8_norm;

impl AttributeFormatIdentifier for Float2_i8_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float2_i8_norm;
}

pub struct Float2_i16_fixed;

impl AttributeFormatIdentifier for Float2_i16_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float2_i16_fixed;
}

pub struct Float2_i16_norm;

impl AttributeFormatIdentifier for Float2_i16_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float2_i16_norm;
}

pub struct Float2_u8_fixed;

impl AttributeFormatIdentifier for Float2_u8_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float2_u8_fixed;
}

pub struct Float2_u8_norm;

impl AttributeFormatIdentifier for Float2_u8_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float2_u8_norm;
}

pub struct Float2_u16_fixed;

impl AttributeFormatIdentifier for Float2_u16_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float2_u16_fixed;
}

pub struct Float2_u16_norm;

impl AttributeFormatIdentifier for Float2_u16_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float2_u16_norm;
}

pub struct Float3_f32;

impl AttributeFormatIdentifier for Float3_f32 {
    const FORMAT: AttributeFormat = AttributeFormat::Float3_f32;
}

pub struct Float3_i8_fixed;

impl AttributeFormatIdentifier for Float3_i8_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float3_i8_fixed;
}

pub struct Float3_i8_norm;

impl AttributeFormatIdentifier for Float3_i8_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float3_i8_norm;
}

pub struct Float3_i16_fixed;

impl AttributeFormatIdentifier for Float3_i16_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float3_i16_fixed;
}

pub struct Float3_i16_norm;

impl AttributeFormatIdentifier for Float3_i16_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float3_i16_norm;
}

pub struct Float3_u8_fixed;

impl AttributeFormatIdentifier for Float3_u8_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float3_u8_fixed;
}

pub struct Float3_u8_norm;

impl AttributeFormatIdentifier for Float3_u8_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float3_u8_norm;
}

pub struct Float3_u16_fixed;

impl AttributeFormatIdentifier for Float3_u16_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float3_u16_fixed;
}

pub struct Float3_u16_norm;

impl AttributeFormatIdentifier for Float3_u16_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float3_u16_norm;
}

pub struct Float4_f32;

impl AttributeFormatIdentifier for Float4_f32 {
    const FORMAT: AttributeFormat = AttributeFormat::Float4_f32;
}

pub struct Float4_i8_fixed;

impl AttributeFormatIdentifier for Float4_i8_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float4_i8_fixed;
}

pub struct Float4_i8_norm;

impl AttributeFormatIdentifier for Float4_i8_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float4_i8_norm;
}

pub struct Float4_i16_fixed;

impl AttributeFormatIdentifier for Float4_i16_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float4_i16_fixed;
}

pub struct Float4_i16_norm;

impl AttributeFormatIdentifier for Float4_i16_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float4_i16_norm;
}

pub struct Float4_u8_fixed;

impl AttributeFormatIdentifier for Float4_u8_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float4_u8_fixed;
}

pub struct Float4_u8_norm;

impl AttributeFormatIdentifier for Float4_u8_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float4_u8_norm;
}

pub struct Float4_u16_fixed;

impl AttributeFormatIdentifier for Float4_u16_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float4_u16_fixed;
}

pub struct Float4_u16_norm;

impl AttributeFormatIdentifier for Float4_u16_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float4_u16_norm;
}

pub struct Float2x2_f32;

impl AttributeFormatIdentifier for Float2x2_f32 {
    const FORMAT: AttributeFormat = AttributeFormat::Float2x2_f32;
}

pub struct Float2x2_i8_fixed;

impl AttributeFormatIdentifier for Float2x2_i8_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float2x2_i8_fixed;
}

pub struct Float2x2_i8_norm;

impl AttributeFormatIdentifier for Float2x2_i8_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float2x2_i8_norm;
}

pub struct Float2x2_i16_fixed;

impl AttributeFormatIdentifier for Float2x2_i16_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float2x2_i16_fixed;
}

pub struct Float2x2_i16_norm;

impl AttributeFormatIdentifier for Float2x2_i16_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float2x2_i16_norm;
}

pub struct Float2x2_u8_fixed;

impl AttributeFormatIdentifier for Float2x2_u8_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float2x2_u8_fixed;
}

pub struct Float2x2_u8_norm;

impl AttributeFormatIdentifier for Float2x2_u8_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float2x2_u8_norm;
}

pub struct Float2x2_u16_fixed;

impl AttributeFormatIdentifier for Float2x2_u16_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float2x2_u16_fixed;
}

pub struct Float2x2_u16_norm;

impl AttributeFormatIdentifier for Float2x2_u16_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float2x2_u16_norm;
}

pub struct Float2x3_f32;

impl AttributeFormatIdentifier for Float2x3_f32 {
    const FORMAT: AttributeFormat = AttributeFormat::Float2x3_f32;
}

pub struct Float2x3_i8_fixed;

impl AttributeFormatIdentifier for Float2x3_i8_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float2x3_i8_fixed;
}

pub struct Float2x3_i8_norm;

impl AttributeFormatIdentifier for Float2x3_i8_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float2x3_i8_norm;
}

pub struct Float2x3_i16_fixed;

impl AttributeFormatIdentifier for Float2x3_i16_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float2x3_i16_fixed;
}

pub struct Float2x3_i16_norm;

impl AttributeFormatIdentifier for Float2x3_i16_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float2x3_i16_norm;
}

pub struct Float2x3_u8_fixed;

impl AttributeFormatIdentifier for Float2x3_u8_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float2x3_u8_fixed;
}

pub struct Float2x3_u8_norm;

impl AttributeFormatIdentifier for Float2x3_u8_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float2x3_u8_norm;
}

pub struct Float2x3_u16_fixed;

impl AttributeFormatIdentifier for Float2x3_u16_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float2x3_u16_fixed;
}

pub struct Float2x3_u16_norm;

impl AttributeFormatIdentifier for Float2x3_u16_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float2x3_u16_norm;
}

pub struct Float2x4_f32;

impl AttributeFormatIdentifier for Float2x4_f32 {
    const FORMAT: AttributeFormat = AttributeFormat::Float2x4_f32;
}

pub struct Float2x4_i8_fixed;

impl AttributeFormatIdentifier for Float2x4_i8_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float2x4_i8_fixed;
}

pub struct Float2x4_i8_norm;

impl AttributeFormatIdentifier for Float2x4_i8_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float2x4_i8_norm;
}

pub struct Float2x4_i16_fixed;

impl AttributeFormatIdentifier for Float2x4_i16_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float2x4_i16_fixed;
}

pub struct Float2x4_i16_norm;

impl AttributeFormatIdentifier for Float2x4_i16_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float2x4_i16_norm;
}

pub struct Float2x4_u8_fixed;

impl AttributeFormatIdentifier for Float2x4_u8_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float2x4_u8_fixed;
}

pub struct Float2x4_u8_norm;

impl AttributeFormatIdentifier for Float2x4_u8_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float2x4_u8_norm;
}

pub struct Float2x4_u16_fixed;

impl AttributeFormatIdentifier for Float2x4_u16_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float2x4_u16_fixed;
}

pub struct Float2x4_u16_norm;

impl AttributeFormatIdentifier for Float2x4_u16_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float2x4_u16_norm;
}

pub struct Float3x2_f32;

impl AttributeFormatIdentifier for Float3x2_f32 {
    const FORMAT: AttributeFormat = AttributeFormat::Float3x2_f32;
}

pub struct Float3x2_i8_fixed;

impl AttributeFormatIdentifier for Float3x2_i8_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float3x2_i8_fixed;
}

pub struct Float3x2_i8_norm;

impl AttributeFormatIdentifier for Float3x2_i8_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float3x2_i8_norm;
}

pub struct Float3x2_i16_fixed;

impl AttributeFormatIdentifier for Float3x2_i16_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float3x2_i16_fixed;
}

pub struct Float3x2_i16_norm;

impl AttributeFormatIdentifier for Float3x2_i16_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float3x2_i16_norm;
}

pub struct Float3x2_u8_fixed;

impl AttributeFormatIdentifier for Float3x2_u8_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float3x2_u8_fixed;
}

pub struct Float3x2_u8_norm;

impl AttributeFormatIdentifier for Float3x2_u8_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float3x2_u8_norm;
}

pub struct Float3x2_u16_fixed;

impl AttributeFormatIdentifier for Float3x2_u16_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float3x2_u16_fixed;
}

pub struct Float3x2_u16_norm;

impl AttributeFormatIdentifier for Float3x2_u16_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float3x2_u16_norm;
}

pub struct Float3x3_f32;

impl AttributeFormatIdentifier for Float3x3_f32 {
    const FORMAT: AttributeFormat = AttributeFormat::Float3x3_f32;
}

pub struct Float3x3_i8_fixed;

impl AttributeFormatIdentifier for Float3x3_i8_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float3x3_i8_fixed;
}

pub struct Float3x3_i8_norm;

impl AttributeFormatIdentifier for Float3x3_i8_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float3x3_i8_norm;
}

pub struct Float3x3_i16_fixed;

impl AttributeFormatIdentifier for Float3x3_i16_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float3x3_i16_fixed;
}

pub struct Float3x3_i16_norm;

impl AttributeFormatIdentifier for Float3x3_i16_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float3x3_i16_norm;
}

pub struct Float3x3_u8_fixed;

impl AttributeFormatIdentifier for Float3x3_u8_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float3x3_u8_fixed;
}

pub struct Float3x3_u8_norm;

impl AttributeFormatIdentifier for Float3x3_u8_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float3x3_u8_norm;
}

pub struct Float3x3_u16_fixed;

impl AttributeFormatIdentifier for Float3x3_u16_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float3x3_u16_fixed;
}

pub struct Float3x3_u16_norm;

impl AttributeFormatIdentifier for Float3x3_u16_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float3x3_u16_norm;
}

pub struct Float3x4_f32;

impl AttributeFormatIdentifier for Float3x4_f32 {
    const FORMAT: AttributeFormat = AttributeFormat::Float3x4_f32;
}

pub struct Float3x4_i8_fixed;

impl AttributeFormatIdentifier for Float3x4_i8_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float3x4_i8_fixed;
}

pub struct Float3x4_i8_norm;

impl AttributeFormatIdentifier for Float3x4_i8_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float3x4_i8_norm;
}

pub struct Float3x4_i16_fixed;

impl AttributeFormatIdentifier for Float3x4_i16_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float3x4_i16_fixed;
}

pub struct Float3x4_i16_norm;

impl AttributeFormatIdentifier for Float3x4_i16_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float3x4_i16_norm;
}

pub struct Float3x4_u8_fixed;

impl AttributeFormatIdentifier for Float3x4_u8_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float3x4_u8_fixed;
}

pub struct Float3x4_u8_norm;

impl AttributeFormatIdentifier for Float3x4_u8_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float3x4_u8_norm;
}

pub struct Float3x4_u16_fixed;

impl AttributeFormatIdentifier for Float3x4_u16_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float3x4_u16_fixed;
}

pub struct Float3x4_u16_norm;

impl AttributeFormatIdentifier for Float3x4_u16_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float3x4_u16_norm;
}

pub struct Float4x2_f32;

impl AttributeFormatIdentifier for Float4x2_f32 {
    const FORMAT: AttributeFormat = AttributeFormat::Float4x2_f32;
}

pub struct Float4x2_i8_fixed;

impl AttributeFormatIdentifier for Float4x2_i8_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float4x2_i8_fixed;
}

pub struct Float4x2_i8_norm;

impl AttributeFormatIdentifier for Float4x2_i8_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float4x2_i8_norm;
}

pub struct Float4x2_i16_fixed;

impl AttributeFormatIdentifier for Float4x2_i16_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float4x2_i16_fixed;
}

pub struct Float4x2_i16_norm;

impl AttributeFormatIdentifier for Float4x2_i16_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float4x2_i16_norm;
}

pub struct Float4x2_u8_fixed;

impl AttributeFormatIdentifier for Float4x2_u8_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float4x2_u8_fixed;
}

pub struct Float4x2_u8_norm;

impl AttributeFormatIdentifier for Float4x2_u8_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float4x2_u8_norm;
}

pub struct Float4x2_u16_fixed;

impl AttributeFormatIdentifier for Float4x2_u16_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float4x2_u16_fixed;
}

pub struct Float4x2_u16_norm;

impl AttributeFormatIdentifier for Float4x2_u16_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float4x2_u16_norm;
}

pub struct Float4x3_f32;

impl AttributeFormatIdentifier for Float4x3_f32 {
    const FORMAT: AttributeFormat = AttributeFormat::Float4x3_f32;
}

pub struct Float4x3_i8_fixed;

impl AttributeFormatIdentifier for Float4x3_i8_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float4x3_i8_fixed;
}

pub struct Float4x3_i8_norm;

impl AttributeFormatIdentifier for Float4x3_i8_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float4x3_i8_norm;
}

pub struct Float4x3_i16_fixed;

impl AttributeFormatIdentifier for Float4x3_i16_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float4x3_i16_fixed;
}

pub struct Float4x3_i16_norm;

impl AttributeFormatIdentifier for Float4x3_i16_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float4x3_i16_norm;
}

pub struct Float4x3_u8_fixed;

impl AttributeFormatIdentifier for Float4x3_u8_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float4x3_u8_fixed;
}

pub struct Float4x3_u8_norm;

impl AttributeFormatIdentifier for Float4x3_u8_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float4x3_u8_norm;
}

pub struct Float4x3_u16_fixed;

impl AttributeFormatIdentifier for Float4x3_u16_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float4x3_u16_fixed;
}

pub struct Float4x3_u16_norm;

impl AttributeFormatIdentifier for Float4x3_u16_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float4x3_u16_norm;
}

pub struct Float4x4_f32;

impl AttributeFormatIdentifier for Float4x4_f32 {
    const FORMAT: AttributeFormat = AttributeFormat::Float4x4_f32;
}

pub struct Float4x4_i8_fixed;

impl AttributeFormatIdentifier for Float4x4_i8_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float4x4_i8_fixed;
}

pub struct Float4x4_i8_norm;

impl AttributeFormatIdentifier for Float4x4_i8_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float4x4_i8_norm;
}

pub struct Float4x4_i16_fixed;

impl AttributeFormatIdentifier for Float4x4_i16_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float4x4_i16_fixed;
}

pub struct Float4x4_i16_norm;

impl AttributeFormatIdentifier for Float4x4_i16_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float4x4_i16_norm;
}

pub struct Float4x4_u8_fixed;

impl AttributeFormatIdentifier for Float4x4_u8_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float4x4_u8_fixed;
}

pub struct Float4x4_u8_norm;

impl AttributeFormatIdentifier for Float4x4_u8_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float4x4_u8_norm;
}

pub struct Float4x4_u16_fixed;

impl AttributeFormatIdentifier for Float4x4_u16_fixed {
    const FORMAT: AttributeFormat = AttributeFormat::Float4x4_u16_fixed;
}

pub struct Float4x4_u16_norm;

impl AttributeFormatIdentifier for Float4x4_u16_norm {
    const FORMAT: AttributeFormat = AttributeFormat::Float4x4_u16_norm;
}

pub struct Integer_i8;

impl AttributeFormatIdentifier for Integer_i8 {
    const FORMAT: AttributeFormat = AttributeFormat::Integer_i8;
}

pub struct Integer_u8;

impl AttributeFormatIdentifier for Integer_u8 {
    const FORMAT: AttributeFormat = AttributeFormat::Integer_u8;
}

pub struct Integer_i16;

impl AttributeFormatIdentifier for Integer_i16 {
    const FORMAT: AttributeFormat = AttributeFormat::Integer_i16;
}

pub struct Integer_u16;

impl AttributeFormatIdentifier for Integer_u16 {
    const FORMAT: AttributeFormat = AttributeFormat::Integer_u16;
}

pub struct Integer_i32;

impl AttributeFormatIdentifier for Integer_i32 {
    const FORMAT: AttributeFormat = AttributeFormat::Integer_i32;
}

pub struct Integer_u32;

impl AttributeFormatIdentifier for Integer_u32 {
    const FORMAT: AttributeFormat = AttributeFormat::Integer_u32;
}

pub struct Integer2_i8;

impl AttributeFormatIdentifier for Integer2_i8 {
    const FORMAT: AttributeFormat = AttributeFormat::Integer2_i8;
}

pub struct Integer2_u8;

impl AttributeFormatIdentifier for Integer2_u8 {
    const FORMAT: AttributeFormat = AttributeFormat::Integer2_u8;
}

pub struct Integer2_i16;

impl AttributeFormatIdentifier for Integer2_i16 {
    const FORMAT: AttributeFormat = AttributeFormat::Integer2_i16;
}

pub struct Integer2_u16;

impl AttributeFormatIdentifier for Integer2_u16 {
    const FORMAT: AttributeFormat = AttributeFormat::Integer2_u16;
}

pub struct Integer2_i32;

impl AttributeFormatIdentifier for Integer2_i32 {
    const FORMAT: AttributeFormat = AttributeFormat::Integer2_i32;
}

pub struct Integer2_u32;

impl AttributeFormatIdentifier for Integer2_u32 {
    const FORMAT: AttributeFormat = AttributeFormat::Integer2_u32;
}

pub struct Integer3_i8;

impl AttributeFormatIdentifier for Integer3_i8 {
    const FORMAT: AttributeFormat = AttributeFormat::Integer3_i8;
}

pub struct Integer3_u8;

impl AttributeFormatIdentifier for Integer3_u8 {
    const FORMAT: AttributeFormat = AttributeFormat::Integer3_u8;
}

pub struct Integer3_i16;

impl AttributeFormatIdentifier for Integer3_i16 {
    const FORMAT: AttributeFormat = AttributeFormat::Integer3_i16;
}

pub struct Integer3_u16;

impl AttributeFormatIdentifier for Integer3_u16 {
    const FORMAT: AttributeFormat = AttributeFormat::Integer3_u16;
}

pub struct Integer3_i32;

impl AttributeFormatIdentifier for Integer3_i32 {
    const FORMAT: AttributeFormat = AttributeFormat::Integer3_i32;
}

pub struct Integer3_u32;

impl AttributeFormatIdentifier for Integer3_u32 {
    const FORMAT: AttributeFormat = AttributeFormat::Integer3_u32;
}

pub struct Integer4_i8;

impl AttributeFormatIdentifier for Integer4_i8 {
    const FORMAT: AttributeFormat = AttributeFormat::Integer4_i8;
}

pub struct Integer4_u8;

impl AttributeFormatIdentifier for Integer4_u8 {
    const FORMAT: AttributeFormat = AttributeFormat::Integer4_u8;
}

pub struct Integer4_i16;

impl AttributeFormatIdentifier for Integer4_i16 {
    const FORMAT: AttributeFormat = AttributeFormat::Integer4_i16;
}

pub struct Integer4_u16;

impl AttributeFormatIdentifier for Integer4_u16 {
    const FORMAT: AttributeFormat = AttributeFormat::Integer4_u16;
}

pub struct Integer4_i32;

impl AttributeFormatIdentifier for Integer4_i32 {
    const FORMAT: AttributeFormat = AttributeFormat::Integer4_i32;
}

pub struct Integer4_u32;

impl AttributeFormatIdentifier for Integer4_u32 {
    const FORMAT: AttributeFormat = AttributeFormat::Integer4_u32;
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
pub unsafe trait FormatCompatible<F>
where
    F: AttributeFormatIdentifier,
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

/// Enumerates all available attribute memory formats.
#[derive(Clone, Copy, PartialEq, Hash, Debug)]
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

impl AttributeFormat {
    /// Whether or not this [AttributeFormat] is compatible with an [AttributeSlotDescriptor] of
    /// the given [AttributeType].
    pub fn is_compatible(&self, attribute_type: AttributeType) -> bool {
        match self {
            AttributeFormat::Float_f32 => attribute_type == AttributeType::Float,
            AttributeFormat::Float_i8_fixed => attribute_type == AttributeType::Float,
            AttributeFormat::Float_i8_norm => attribute_type == AttributeType::Float,
            AttributeFormat::Float_i16_fixed => attribute_type == AttributeType::Float,
            AttributeFormat::Float_i16_norm => attribute_type == AttributeType::Float,
            AttributeFormat::Float_u8_fixed => attribute_type == AttributeType::Float,
            AttributeFormat::Float_u8_norm => attribute_type == AttributeType::Float,
            AttributeFormat::Float_u16_fixed => attribute_type == AttributeType::Float,
            AttributeFormat::Float_u16_norm => attribute_type == AttributeType::Float,
            AttributeFormat::Float2_f32 => attribute_type == AttributeType::FloatVector2,
            AttributeFormat::Float2_i8_fixed => attribute_type == AttributeType::FloatVector2,
            AttributeFormat::Float2_i8_norm => attribute_type == AttributeType::FloatVector2,
            AttributeFormat::Float2_i16_fixed => attribute_type == AttributeType::FloatVector2,
            AttributeFormat::Float2_i16_norm => attribute_type == AttributeType::FloatVector2,
            AttributeFormat::Float2_u8_fixed => attribute_type == AttributeType::FloatVector2,
            AttributeFormat::Float2_u8_norm => attribute_type == AttributeType::FloatVector2,
            AttributeFormat::Float2_u16_fixed => attribute_type == AttributeType::FloatVector2,
            AttributeFormat::Float2_u16_norm => attribute_type == AttributeType::FloatVector2,
            AttributeFormat::Float3_f32 => attribute_type == AttributeType::FloatVector3,
            AttributeFormat::Float3_i8_fixed => attribute_type == AttributeType::FloatVector3,
            AttributeFormat::Float3_i8_norm => attribute_type == AttributeType::FloatVector3,
            AttributeFormat::Float3_i16_fixed => attribute_type == AttributeType::FloatVector3,
            AttributeFormat::Float3_i16_norm => attribute_type == AttributeType::FloatVector3,
            AttributeFormat::Float3_u8_fixed => attribute_type == AttributeType::FloatVector3,
            AttributeFormat::Float3_u8_norm => attribute_type == AttributeType::FloatVector3,
            AttributeFormat::Float3_u16_fixed => attribute_type == AttributeType::FloatVector3,
            AttributeFormat::Float3_u16_norm => attribute_type == AttributeType::FloatVector3,
            AttributeFormat::Float4_f32 => attribute_type == AttributeType::FloatVector4,
            AttributeFormat::Float4_i8_fixed => attribute_type == AttributeType::FloatVector4,
            AttributeFormat::Float4_i8_norm => attribute_type == AttributeType::FloatVector4,
            AttributeFormat::Float4_i16_fixed => attribute_type == AttributeType::FloatVector4,
            AttributeFormat::Float4_i16_norm => attribute_type == AttributeType::FloatVector4,
            AttributeFormat::Float4_u8_fixed => attribute_type == AttributeType::FloatVector4,
            AttributeFormat::Float4_u8_norm => attribute_type == AttributeType::FloatVector4,
            AttributeFormat::Float4_u16_fixed => attribute_type == AttributeType::FloatVector4,
            AttributeFormat::Float4_u16_norm => attribute_type == AttributeType::FloatVector4,
            AttributeFormat::Float2x2_f32 => attribute_type == AttributeType::FloatMatrix2x2,
            AttributeFormat::Float2x2_i8_fixed => attribute_type == AttributeType::FloatMatrix2x2,
            AttributeFormat::Float2x2_i8_norm => attribute_type == AttributeType::FloatMatrix2x2,
            AttributeFormat::Float2x2_i16_fixed => attribute_type == AttributeType::FloatMatrix2x2,
            AttributeFormat::Float2x2_i16_norm => attribute_type == AttributeType::FloatMatrix2x2,
            AttributeFormat::Float2x2_u8_fixed => attribute_type == AttributeType::FloatMatrix2x2,
            AttributeFormat::Float2x2_u8_norm => attribute_type == AttributeType::FloatMatrix2x2,
            AttributeFormat::Float2x2_u16_fixed => attribute_type == AttributeType::FloatMatrix2x2,
            AttributeFormat::Float2x2_u16_norm => attribute_type == AttributeType::FloatMatrix2x2,
            AttributeFormat::Float2x3_f32 => attribute_type == AttributeType::FloatMatrix2x3,
            AttributeFormat::Float2x3_i8_fixed => attribute_type == AttributeType::FloatMatrix2x3,
            AttributeFormat::Float2x3_i8_norm => attribute_type == AttributeType::FloatMatrix2x3,
            AttributeFormat::Float2x3_i16_fixed => attribute_type == AttributeType::FloatMatrix2x3,
            AttributeFormat::Float2x3_i16_norm => attribute_type == AttributeType::FloatMatrix2x3,
            AttributeFormat::Float2x3_u8_fixed => attribute_type == AttributeType::FloatMatrix2x3,
            AttributeFormat::Float2x3_u8_norm => attribute_type == AttributeType::FloatMatrix2x3,
            AttributeFormat::Float2x3_u16_fixed => attribute_type == AttributeType::FloatMatrix2x3,
            AttributeFormat::Float2x3_u16_norm => attribute_type == AttributeType::FloatMatrix2x3,
            AttributeFormat::Float2x4_f32 => attribute_type == AttributeType::FloatMatrix2x4,
            AttributeFormat::Float2x4_i8_fixed => attribute_type == AttributeType::FloatMatrix2x4,
            AttributeFormat::Float2x4_i8_norm => attribute_type == AttributeType::FloatMatrix2x4,
            AttributeFormat::Float2x4_i16_fixed => attribute_type == AttributeType::FloatMatrix2x4,
            AttributeFormat::Float2x4_i16_norm => attribute_type == AttributeType::FloatMatrix2x4,
            AttributeFormat::Float2x4_u8_fixed => attribute_type == AttributeType::FloatMatrix2x4,
            AttributeFormat::Float2x4_u8_norm => attribute_type == AttributeType::FloatMatrix2x4,
            AttributeFormat::Float2x4_u16_fixed => attribute_type == AttributeType::FloatMatrix2x4,
            AttributeFormat::Float2x4_u16_norm => attribute_type == AttributeType::FloatMatrix2x4,
            AttributeFormat::Float3x2_f32 => attribute_type == AttributeType::FloatMatrix3x2,
            AttributeFormat::Float3x2_i8_fixed => attribute_type == AttributeType::FloatMatrix3x2,
            AttributeFormat::Float3x2_i8_norm => attribute_type == AttributeType::FloatMatrix3x2,
            AttributeFormat::Float3x2_i16_fixed => attribute_type == AttributeType::FloatMatrix3x2,
            AttributeFormat::Float3x2_i16_norm => attribute_type == AttributeType::FloatMatrix3x2,
            AttributeFormat::Float3x2_u8_fixed => attribute_type == AttributeType::FloatMatrix3x2,
            AttributeFormat::Float3x2_u8_norm => attribute_type == AttributeType::FloatMatrix3x2,
            AttributeFormat::Float3x2_u16_fixed => attribute_type == AttributeType::FloatMatrix3x2,
            AttributeFormat::Float3x2_u16_norm => attribute_type == AttributeType::FloatMatrix3x2,
            AttributeFormat::Float3x3_f32 => attribute_type == AttributeType::FloatMatrix3x3,
            AttributeFormat::Float3x3_i8_fixed => attribute_type == AttributeType::FloatMatrix3x3,
            AttributeFormat::Float3x3_i8_norm => attribute_type == AttributeType::FloatMatrix3x3,
            AttributeFormat::Float3x3_i16_fixed => attribute_type == AttributeType::FloatMatrix3x3,
            AttributeFormat::Float3x3_i16_norm => attribute_type == AttributeType::FloatMatrix3x3,
            AttributeFormat::Float3x3_u8_fixed => attribute_type == AttributeType::FloatMatrix3x3,
            AttributeFormat::Float3x3_u8_norm => attribute_type == AttributeType::FloatMatrix3x3,
            AttributeFormat::Float3x3_u16_fixed => attribute_type == AttributeType::FloatMatrix3x3,
            AttributeFormat::Float3x3_u16_norm => attribute_type == AttributeType::FloatMatrix3x3,
            AttributeFormat::Float3x4_f32 => attribute_type == AttributeType::FloatMatrix3x4,
            AttributeFormat::Float3x4_i8_fixed => attribute_type == AttributeType::FloatMatrix3x4,
            AttributeFormat::Float3x4_i8_norm => attribute_type == AttributeType::FloatMatrix3x4,
            AttributeFormat::Float3x4_i16_fixed => attribute_type == AttributeType::FloatMatrix3x4,
            AttributeFormat::Float3x4_i16_norm => attribute_type == AttributeType::FloatMatrix3x4,
            AttributeFormat::Float3x4_u8_fixed => attribute_type == AttributeType::FloatMatrix3x4,
            AttributeFormat::Float3x4_u8_norm => attribute_type == AttributeType::FloatMatrix3x4,
            AttributeFormat::Float3x4_u16_fixed => attribute_type == AttributeType::FloatMatrix3x4,
            AttributeFormat::Float3x4_u16_norm => attribute_type == AttributeType::FloatMatrix3x4,
            AttributeFormat::Float4x2_f32 => attribute_type == AttributeType::FloatMatrix4x2,
            AttributeFormat::Float4x2_i8_fixed => attribute_type == AttributeType::FloatMatrix4x2,
            AttributeFormat::Float4x2_i8_norm => attribute_type == AttributeType::FloatMatrix4x2,
            AttributeFormat::Float4x2_i16_fixed => attribute_type == AttributeType::FloatMatrix4x2,
            AttributeFormat::Float4x2_i16_norm => attribute_type == AttributeType::FloatMatrix4x2,
            AttributeFormat::Float4x2_u8_fixed => attribute_type == AttributeType::FloatMatrix4x2,
            AttributeFormat::Float4x2_u8_norm => attribute_type == AttributeType::FloatMatrix4x2,
            AttributeFormat::Float4x2_u16_fixed => attribute_type == AttributeType::FloatMatrix4x2,
            AttributeFormat::Float4x2_u16_norm => attribute_type == AttributeType::FloatMatrix4x2,
            AttributeFormat::Float4x3_f32 => attribute_type == AttributeType::FloatMatrix4x3,
            AttributeFormat::Float4x3_i8_fixed => attribute_type == AttributeType::FloatMatrix4x3,
            AttributeFormat::Float4x3_i8_norm => attribute_type == AttributeType::FloatMatrix4x3,
            AttributeFormat::Float4x3_i16_fixed => attribute_type == AttributeType::FloatMatrix4x3,
            AttributeFormat::Float4x3_i16_norm => attribute_type == AttributeType::FloatMatrix4x3,
            AttributeFormat::Float4x3_u8_fixed => attribute_type == AttributeType::FloatMatrix4x3,
            AttributeFormat::Float4x3_u8_norm => attribute_type == AttributeType::FloatMatrix4x3,
            AttributeFormat::Float4x3_u16_fixed => attribute_type == AttributeType::FloatMatrix4x3,
            AttributeFormat::Float4x3_u16_norm => attribute_type == AttributeType::FloatMatrix4x3,
            AttributeFormat::Float4x4_f32 => attribute_type == AttributeType::FloatMatrix4x4,
            AttributeFormat::Float4x4_i8_fixed => attribute_type == AttributeType::FloatMatrix4x4,
            AttributeFormat::Float4x4_i8_norm => attribute_type == AttributeType::FloatMatrix4x4,
            AttributeFormat::Float4x4_i16_fixed => attribute_type == AttributeType::FloatMatrix4x4,
            AttributeFormat::Float4x4_i16_norm => attribute_type == AttributeType::FloatMatrix4x4,
            AttributeFormat::Float4x4_u8_fixed => attribute_type == AttributeType::FloatMatrix4x4,
            AttributeFormat::Float4x4_u8_norm => attribute_type == AttributeType::FloatMatrix4x4,
            AttributeFormat::Float4x4_u16_fixed => attribute_type == AttributeType::FloatMatrix4x4,
            AttributeFormat::Float4x4_u16_norm => attribute_type == AttributeType::FloatMatrix4x4,
            AttributeFormat::Integer_i8 => attribute_type == AttributeType::Integer,
            AttributeFormat::Integer_u8 => attribute_type == AttributeType::UnsignedInteger,
            AttributeFormat::Integer_i16 => attribute_type == AttributeType::Integer,
            AttributeFormat::Integer_u16 => attribute_type == AttributeType::UnsignedInteger,
            AttributeFormat::Integer_i32 => attribute_type == AttributeType::Integer,
            AttributeFormat::Integer_u32 => attribute_type == AttributeType::UnsignedInteger,
            AttributeFormat::Integer2_i8 => attribute_type == AttributeType::IntegerVector2,
            AttributeFormat::Integer2_u8 => attribute_type == AttributeType::UnsignedIntegerVector2,
            AttributeFormat::Integer2_i16 => attribute_type == AttributeType::IntegerVector2,
            AttributeFormat::Integer2_u16 => {
                attribute_type == AttributeType::UnsignedIntegerVector2
            }
            AttributeFormat::Integer2_i32 => attribute_type == AttributeType::IntegerVector2,
            AttributeFormat::Integer2_u32 => {
                attribute_type == AttributeType::UnsignedIntegerVector2
            }
            AttributeFormat::Integer3_i8 => attribute_type == AttributeType::IntegerVector3,
            AttributeFormat::Integer3_u8 => attribute_type == AttributeType::UnsignedIntegerVector3,
            AttributeFormat::Integer3_i16 => attribute_type == AttributeType::IntegerVector3,
            AttributeFormat::Integer3_u16 => {
                attribute_type == AttributeType::UnsignedIntegerVector3
            }
            AttributeFormat::Integer3_i32 => attribute_type == AttributeType::IntegerVector3,
            AttributeFormat::Integer3_u32 => {
                attribute_type == AttributeType::UnsignedIntegerVector3
            }
            AttributeFormat::Integer4_i8 => attribute_type == AttributeType::IntegerVector4,
            AttributeFormat::Integer4_u8 => attribute_type == AttributeType::UnsignedIntegerVector4,
            AttributeFormat::Integer4_i16 => attribute_type == AttributeType::IntegerVector4,
            AttributeFormat::Integer4_u16 => {
                attribute_type == AttributeType::UnsignedIntegerVector4
            }
            AttributeFormat::Integer4_i32 => attribute_type == AttributeType::IntegerVector4,
            AttributeFormat::Integer4_u32 => {
                attribute_type == AttributeType::UnsignedIntegerVector4
            }
        }
    }

    pub fn size_in_bytes(&self) -> u8 {
        match self {
            AttributeFormat::Float_f32 => 4,
            AttributeFormat::Float_i8_fixed => 1,
            AttributeFormat::Float_i8_norm => 1,
            AttributeFormat::Float_i16_fixed => 2,
            AttributeFormat::Float_i16_norm => 2,
            AttributeFormat::Float_u8_fixed => 1,
            AttributeFormat::Float_u8_norm => 1,
            AttributeFormat::Float_u16_fixed => 2,
            AttributeFormat::Float_u16_norm => 2,
            AttributeFormat::Float2_f32 => 8,
            AttributeFormat::Float2_i8_fixed => 2,
            AttributeFormat::Float2_i8_norm => 2,
            AttributeFormat::Float2_i16_fixed => 4,
            AttributeFormat::Float2_i16_norm => 4,
            AttributeFormat::Float2_u8_fixed => 2,
            AttributeFormat::Float2_u8_norm => 2,
            AttributeFormat::Float2_u16_fixed => 4,
            AttributeFormat::Float2_u16_norm => 4,
            AttributeFormat::Float3_f32 => 12,
            AttributeFormat::Float3_i8_fixed => 3,
            AttributeFormat::Float3_i8_norm => 3,
            AttributeFormat::Float3_i16_fixed => 6,
            AttributeFormat::Float3_i16_norm => 6,
            AttributeFormat::Float3_u8_fixed => 3,
            AttributeFormat::Float3_u8_norm => 3,
            AttributeFormat::Float3_u16_fixed => 6,
            AttributeFormat::Float3_u16_norm => 6,
            AttributeFormat::Float4_f32 => 16,
            AttributeFormat::Float4_i8_fixed => 4,
            AttributeFormat::Float4_i8_norm => 4,
            AttributeFormat::Float4_i16_fixed => 8,
            AttributeFormat::Float4_i16_norm => 8,
            AttributeFormat::Float4_u8_fixed => 4,
            AttributeFormat::Float4_u8_norm => 4,
            AttributeFormat::Float4_u16_fixed => 8,
            AttributeFormat::Float4_u16_norm => 8,
            AttributeFormat::Float2x2_f32 => 16,
            AttributeFormat::Float2x2_i8_fixed => 4,
            AttributeFormat::Float2x2_i8_norm => 4,
            AttributeFormat::Float2x2_i16_fixed => 8,
            AttributeFormat::Float2x2_i16_norm => 8,
            AttributeFormat::Float2x2_u8_fixed => 4,
            AttributeFormat::Float2x2_u8_norm => 4,
            AttributeFormat::Float2x2_u16_fixed => 8,
            AttributeFormat::Float2x2_u16_norm => 8,
            AttributeFormat::Float2x3_f32 => 24,
            AttributeFormat::Float2x3_i8_fixed => 6,
            AttributeFormat::Float2x3_i8_norm => 6,
            AttributeFormat::Float2x3_i16_fixed => 12,
            AttributeFormat::Float2x3_i16_norm => 12,
            AttributeFormat::Float2x3_u8_fixed => 6,
            AttributeFormat::Float2x3_u8_norm => 6,
            AttributeFormat::Float2x3_u16_fixed => 12,
            AttributeFormat::Float2x3_u16_norm => 12,
            AttributeFormat::Float2x4_f32 => 32,
            AttributeFormat::Float2x4_i8_fixed => 8,
            AttributeFormat::Float2x4_i8_norm => 8,
            AttributeFormat::Float2x4_i16_fixed => 16,
            AttributeFormat::Float2x4_i16_norm => 16,
            AttributeFormat::Float2x4_u8_fixed => 8,
            AttributeFormat::Float2x4_u8_norm => 8,
            AttributeFormat::Float2x4_u16_fixed => 16,
            AttributeFormat::Float2x4_u16_norm => 16,
            AttributeFormat::Float3x2_f32 => 24,
            AttributeFormat::Float3x2_i8_fixed => 6,
            AttributeFormat::Float3x2_i8_norm => 6,
            AttributeFormat::Float3x2_i16_fixed => 12,
            AttributeFormat::Float3x2_i16_norm => 12,
            AttributeFormat::Float3x2_u8_fixed => 6,
            AttributeFormat::Float3x2_u8_norm => 6,
            AttributeFormat::Float3x2_u16_fixed => 12,
            AttributeFormat::Float3x2_u16_norm => 12,
            AttributeFormat::Float3x3_f32 => 36,
            AttributeFormat::Float3x3_i8_fixed => 9,
            AttributeFormat::Float3x3_i8_norm => 9,
            AttributeFormat::Float3x3_i16_fixed => 18,
            AttributeFormat::Float3x3_i16_norm => 18,
            AttributeFormat::Float3x3_u8_fixed => 9,
            AttributeFormat::Float3x3_u8_norm => 9,
            AttributeFormat::Float3x3_u16_fixed => 18,
            AttributeFormat::Float3x3_u16_norm => 18,
            AttributeFormat::Float3x4_f32 => 48,
            AttributeFormat::Float3x4_i8_fixed => 12,
            AttributeFormat::Float3x4_i8_norm => 12,
            AttributeFormat::Float3x4_i16_fixed => 24,
            AttributeFormat::Float3x4_i16_norm => 24,
            AttributeFormat::Float3x4_u8_fixed => 12,
            AttributeFormat::Float3x4_u8_norm => 12,
            AttributeFormat::Float3x4_u16_fixed => 24,
            AttributeFormat::Float3x4_u16_norm => 24,
            AttributeFormat::Float4x2_f32 => 32,
            AttributeFormat::Float4x2_i8_fixed => 8,
            AttributeFormat::Float4x2_i8_norm => 8,
            AttributeFormat::Float4x2_i16_fixed => 16,
            AttributeFormat::Float4x2_i16_norm => 16,
            AttributeFormat::Float4x2_u8_fixed => 8,
            AttributeFormat::Float4x2_u8_norm => 8,
            AttributeFormat::Float4x2_u16_fixed => 16,
            AttributeFormat::Float4x2_u16_norm => 16,
            AttributeFormat::Float4x3_f32 => 48,
            AttributeFormat::Float4x3_i8_fixed => 12,
            AttributeFormat::Float4x3_i8_norm => 12,
            AttributeFormat::Float4x3_i16_fixed => 24,
            AttributeFormat::Float4x3_i16_norm => 24,
            AttributeFormat::Float4x3_u8_fixed => 12,
            AttributeFormat::Float4x3_u8_norm => 12,
            AttributeFormat::Float4x3_u16_fixed => 24,
            AttributeFormat::Float4x3_u16_norm => 24,
            AttributeFormat::Float4x4_f32 => 64,
            AttributeFormat::Float4x4_i8_fixed => 16,
            AttributeFormat::Float4x4_i8_norm => 16,
            AttributeFormat::Float4x4_i16_fixed => 32,
            AttributeFormat::Float4x4_i16_norm => 32,
            AttributeFormat::Float4x4_u8_fixed => 16,
            AttributeFormat::Float4x4_u8_norm => 16,
            AttributeFormat::Float4x4_u16_fixed => 32,
            AttributeFormat::Float4x4_u16_norm => 32,
            AttributeFormat::Integer_i8 => 1,
            AttributeFormat::Integer_u8 => 1,
            AttributeFormat::Integer_i16 => 2,
            AttributeFormat::Integer_u16 => 2,
            AttributeFormat::Integer_i32 => 4,
            AttributeFormat::Integer_u32 => 4,
            AttributeFormat::Integer2_i8 => 2,
            AttributeFormat::Integer2_u8 => 2,
            AttributeFormat::Integer2_i16 => 4,
            AttributeFormat::Integer2_u16 => 4,
            AttributeFormat::Integer2_i32 => 8,
            AttributeFormat::Integer2_u32 => 8,
            AttributeFormat::Integer3_i8 => 3,
            AttributeFormat::Integer3_u8 => 3,
            AttributeFormat::Integer3_i16 => 6,
            AttributeFormat::Integer3_u16 => 6,
            AttributeFormat::Integer3_i32 => 12,
            AttributeFormat::Integer3_u32 => 12,
            AttributeFormat::Integer4_i8 => 4,
            AttributeFormat::Integer4_u8 => 4,
            AttributeFormat::Integer4_i16 => 8,
            AttributeFormat::Integer4_u16 => 8,
            AttributeFormat::Integer4_i32 => 16,
            AttributeFormat::Integer4_u32 => 16,
        }
    }
}
