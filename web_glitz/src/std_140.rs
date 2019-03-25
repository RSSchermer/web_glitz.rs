#![allow(non_camel_case_types)]

use crate::pipeline::interface_block::CheckCompatibility;
use crate::pipeline::interface_block::Incompatible;
use crate::pipeline::interface_block::InterfaceBlock;
use crate::pipeline::interface_block::InterfaceBlockComponent;
use crate::pipeline::interface_block::MemoryUnitDescriptor;
use crate::pipeline::interface_block::UnitLayout;
//
//#[repr(C, align(16))]
//pub struct Test0 {
//    member0: float,
//    member1: vec2,
//}
//
//unsafe impl InterfaceBlock for Test0 {
//    fn compatibility(memory_layout: &[MemoryUnitDescriptor]) -> Result<(), Incompatible> {
//        let mut iter = memory_layout.iter();
//
//        match Test0::check_compatibility(0, &mut iter) {
//            CheckCompatibility::Finished => Ok(()),
//            CheckCompatibility::Continue => {
//                if let Some(unit) = iter.next() {
//                    Err(Incompatible::MissingUnit(unit.clone()))
//                } else {
//                    Ok(())
//                }
//            }
//            CheckCompatibility::Incompatible(incompatible) => Err(incompatible),
//        }
//    }
//}
//
//unsafe impl IntefaceBlockComponent for Test0 {
//    fn check_compatibility<I>(base_offset: usize, remainder: &mut I) -> CheckCompatibility
//    where
//        I: Iterator<Item = &MemoryUnitDescriptor>,
//    {
//        let check_member0 =
//            float::check_compatibility(base_offset + offset_of!(Test0, member0), remainder);
//
//        if check_member0 != CheckCompatibility::Continue {
//            return check_member0;
//        }
//
//        let check_member1 =
//            vec2::check_compatibility(base_offset + offset_of!(Test0, member1), remainder);
//
//        if check_member1 != CheckCompatibility::Continue {
//            return check_member1;
//        }
//
//        CheckCompatibility::Continue
//    }
//}
//
//#[repr(C, align(16))]
//pub struct Test1 {
//    member0: Test0,
//    member1: vec4,
//}
//
//unsafe impl IntefaceBlockComponent for Test1 {
//    fn check_compatibility<I>(base_offset: usize, remainder: &mut I) -> CheckCompatibility
//    where
//        I: Iterator<Item = &MemoryUnitDescriptor>,
//    {
//        let check_member0 =
//            Test0::check_compatibility(base_offset + offset_of!(Test0, member0), remainder);
//
//        if check_member0 != CheckCompatibility::Continue {
//            return check_member0;
//        }
//
//        let check_member1 =
//            vec4::check_compatibility(base_offset + offset_of!(Test0, member1), remainder);
//
//        if check_member1 != CheckCompatibility::Continue {
//            return check_member1;
//        }
//
//        CheckCompatibility::Continue
//    }
//}
//
//unsafe impl InterfaceBlock for Test1 {
//    fn compatibility(memory_layout: &[MemoryUnitDescriptor]) -> Result<(), Incompatible> {
//        let mut iter = memory_layout.iter();
//
//        match Test0::check_compatibility(0, &mut iter) {
//            CheckCompatibility::Finished => Ok(()),
//            CheckCompatibility::Continue => {
//                if let Some(unit) = iter.next() {
//                    Err(Incompatible::MissingUnit(unit.clone()))
//                } else {
//                    Ok(())
//                }
//            }
//            CheckCompatibility::Incompatible(incompatible) => Err(incompatible),
//        }
//    }
//}

pub unsafe trait ReprStd140 {}

pub unsafe trait Std140ArrayElement: ReprStd140 {}

//pub struct array<T, const LEN: usize> where T: Std140ArrayElement {
//    internal: [ArrayElementWrapper<T>: LEN]
//}

#[repr(C, align(16))]
struct ArrayElementWrapper<T>
where
    T: Std140ArrayElement,
{
    element: T,
}

macro_rules! impl_interface_block_component {
    ($T:ident, $layout:expr) => {
        unsafe impl InterfaceBlockComponent for $T {
            fn check_compatibility<'a, I>(
                component_offset: usize,
                remainder: &'a mut I,
            ) -> CheckCompatibility
            where
                I: Iterator<Item = &'a MemoryUnitDescriptor>,
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
pub struct float(f32);

impl_interface_block_component!(float, UnitLayout::Float);
unsafe impl ReprStd140 for float {}
unsafe impl Std140ArrayElement for float {}

#[repr(C, align(8))]
pub struct vec2(f32, f32);

impl_interface_block_component!(vec2, UnitLayout::FloatVector2);
unsafe impl ReprStd140 for vec2 {}
unsafe impl Std140ArrayElement for vec2 {}

#[repr(C, align(16))]
pub struct vec3(f32, f32, f32);

impl_interface_block_component!(vec3, UnitLayout::FloatVector3);
unsafe impl ReprStd140 for vec3 {}
unsafe impl Std140ArrayElement for vec3 {}

#[repr(C, align(16))]
pub struct vec4(f32, f32, f32, f32);

impl_interface_block_component!(vec4, UnitLayout::FloatVector4);
unsafe impl ReprStd140 for vec4 {}
unsafe impl Std140ArrayElement for vec4 {}

#[repr(C, align(4))]
pub struct int(i32);

impl_interface_block_component!(int, UnitLayout::Integer);
unsafe impl ReprStd140 for int {}
unsafe impl Std140ArrayElement for int {}

#[repr(C, align(8))]
pub struct ivec2(i32, i32);

impl_interface_block_component!(ivec2, UnitLayout::IntegerVector2);
unsafe impl ReprStd140 for ivec2 {}
unsafe impl Std140ArrayElement for ivec2 {}

#[repr(C, align(16))]
pub struct ivec3(i32, i32, i32);

impl_interface_block_component!(ivec3, UnitLayout::IntegerVector3);
unsafe impl ReprStd140 for ivec3 {}
unsafe impl Std140ArrayElement for ivec3 {}

#[repr(C, align(16))]
pub struct ivec4(i32, i32, i32, i32);

impl_interface_block_component!(ivec4, UnitLayout::IntegerVector4);
unsafe impl ReprStd140 for ivec4 {}
unsafe impl Std140ArrayElement for ivec4 {}

#[repr(C, align(4))]
pub struct uint(u32);

impl_interface_block_component!(uint, UnitLayout::UnsignedInteger);
unsafe impl ReprStd140 for uint {}
unsafe impl Std140ArrayElement for uint {}

#[repr(C, align(8))]
pub struct uvec2(u32, u32);

impl_interface_block_component!(uvec2, UnitLayout::UnsignedIntegerVector2);
unsafe impl ReprStd140 for uvec2 {}
unsafe impl Std140ArrayElement for uvec2 {}

#[repr(C, align(16))]
pub struct uvec3(u32, u32, u32);

impl_interface_block_component!(uvec3, UnitLayout::UnsignedIntegerVector3);
unsafe impl ReprStd140 for uvec3 {}
unsafe impl Std140ArrayElement for uvec3 {}

#[repr(C, align(16))]
pub struct uvec4(u32, u32, u32, u32);

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
pub struct bvec2(boolean, boolean);

impl_interface_block_component!(bvec2, UnitLayout::BoolVector2);
unsafe impl ReprStd140 for bvec2 {}
unsafe impl Std140ArrayElement for bvec2 {}

#[repr(C, align(16))]
pub struct bvec3(boolean, boolean, boolean);

impl_interface_block_component!(bvec3, UnitLayout::BoolVector3);
unsafe impl ReprStd140 for bvec3 {}
unsafe impl Std140ArrayElement for bvec3 {}

#[repr(C, align(16))]
pub struct bvec4(boolean, boolean, boolean, boolean);

impl_interface_block_component!(bvec4, UnitLayout::BoolVector4);
unsafe impl ReprStd140 for bvec4 {}
unsafe impl Std140ArrayElement for bvec4 {}
