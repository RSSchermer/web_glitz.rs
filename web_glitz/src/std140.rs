#![allow(non_camel_case_types)]

use crate::pipeline::interface_block::{
    CheckCompatibility, Incompatible, InterfaceBlockComponent, MemoryUnitDescriptor, UnitLayout,
};

pub unsafe trait ReprStd140 {}

pub unsafe trait Std140ArrayElement: ReprStd140 {}

//pub struct array<T, const LEN: usize> where T: Std140ArrayElement {
//    internal: [ArrayElementWrapper<T>: LEN]
//}
//
//#[repr(C, align(16))]
//struct ArrayElementWrapper<T>
//where
//    T: Std140ArrayElement,
//{
//    element: T,
//}

macro_rules! impl_interface_block_component {
    ($T:ident, $layout:expr) => {
        unsafe impl InterfaceBlockComponent for $T {
            fn check_compatibility<'a, 'b, I>(
                component_offset: usize,
                remainder: &'a mut I,
            ) -> CheckCompatibility
            where
                I: Iterator<Item = &'b MemoryUnitDescriptor>,
                'b: 'a,
            {
                if let Some(unit) = remainder.next() {
                    if unit.offset() != component_offset {
                        CheckCompatibility::Incompatible(Incompatible::MissingUnit(unit.clone()))
                    } else if unit.layout() != &$layout {
                        CheckCompatibility::Incompatible(Incompatible::UnitLayoutMismatch(
                            unit.clone(),
                            $layout,
                        ))
                    } else {
                        CheckCompatibility::Continue
                    }
                } else {
                    CheckCompatibility::Finished
                }
            }
        }
    };
}

#[repr(C, align(4))]
pub struct float(pub f32);

impl_interface_block_component!(float, UnitLayout::Float);
unsafe impl ReprStd140 for float {}
unsafe impl Std140ArrayElement for float {}

#[repr(C, align(8))]
pub struct vec2(pub f32, pub f32);

impl_interface_block_component!(vec2, UnitLayout::FloatVector2);
unsafe impl ReprStd140 for vec2 {}
unsafe impl Std140ArrayElement for vec2 {}

#[repr(C, align(16))]
pub struct vec3(pub f32, pub f32, pub f32);

impl_interface_block_component!(vec3, UnitLayout::FloatVector3);
unsafe impl ReprStd140 for vec3 {}
unsafe impl Std140ArrayElement for vec3 {}

#[repr(C, align(16))]
pub struct vec4(pub f32, pub f32, pub f32, pub f32);

impl_interface_block_component!(vec4, UnitLayout::FloatVector4);
unsafe impl ReprStd140 for vec4 {}
unsafe impl Std140ArrayElement for vec4 {}

#[repr(C, align(4))]
pub struct int(pub i32);

impl_interface_block_component!(int, UnitLayout::Integer);
unsafe impl ReprStd140 for int {}
unsafe impl Std140ArrayElement for int {}

#[repr(C, align(8))]
pub struct ivec2(pub i32, pub i32);

impl_interface_block_component!(ivec2, UnitLayout::IntegerVector2);
unsafe impl ReprStd140 for ivec2 {}
unsafe impl Std140ArrayElement for ivec2 {}

#[repr(C, align(16))]
pub struct ivec3(pub i32, pub i32, pub i32);

impl_interface_block_component!(ivec3, UnitLayout::IntegerVector3);
unsafe impl ReprStd140 for ivec3 {}
unsafe impl Std140ArrayElement for ivec3 {}

#[repr(C, align(16))]
pub struct ivec4(pub i32, pub i32, pub i32, pub i32);

impl_interface_block_component!(ivec4, UnitLayout::IntegerVector4);
unsafe impl ReprStd140 for ivec4 {}
unsafe impl Std140ArrayElement for ivec4 {}

#[repr(C, align(4))]
pub struct uint(pub u32);

impl_interface_block_component!(uint, UnitLayout::UnsignedInteger);
unsafe impl ReprStd140 for uint {}
unsafe impl Std140ArrayElement for uint {}

#[repr(C, align(8))]
pub struct uvec2(pub u32, pub u32);

impl_interface_block_component!(uvec2, UnitLayout::UnsignedIntegerVector2);
unsafe impl ReprStd140 for uvec2 {}
unsafe impl Std140ArrayElement for uvec2 {}

#[repr(C, align(16))]
pub struct uvec3(pub u32, pub u32, pub u32);

impl_interface_block_component!(uvec3, UnitLayout::UnsignedIntegerVector3);
unsafe impl ReprStd140 for uvec3 {}
unsafe impl Std140ArrayElement for uvec3 {}

#[repr(C, align(16))]
pub struct uvec4(pub u32, pub u32, pub u32, pub u32);

impl_interface_block_component!(uvec4, UnitLayout::UnsignedIntegerVector4);
unsafe impl ReprStd140 for uvec4 {}
unsafe impl Std140ArrayElement for uvec4 {}

#[repr(u32)]
pub enum boolean {
    True = 1,
    False = 0,
}

impl_interface_block_component!(boolean, UnitLayout::Bool);
unsafe impl ReprStd140 for boolean {}
unsafe impl Std140ArrayElement for boolean {}

#[repr(C, align(8))]
pub struct bvec2(pub boolean, pub boolean);

impl_interface_block_component!(bvec2, UnitLayout::BoolVector2);
unsafe impl ReprStd140 for bvec2 {}
unsafe impl Std140ArrayElement for bvec2 {}

#[repr(C, align(16))]
pub struct bvec3(pub boolean, pub boolean, pub boolean);

impl_interface_block_component!(bvec3, UnitLayout::BoolVector3);
unsafe impl ReprStd140 for bvec3 {}
unsafe impl Std140ArrayElement for bvec3 {}

#[repr(C, align(16))]
pub struct bvec4(pub boolean, pub boolean, pub boolean, pub boolean);

impl_interface_block_component!(bvec4, UnitLayout::BoolVector4);
unsafe impl ReprStd140 for bvec4 {}
unsafe impl Std140ArrayElement for bvec4 {}
