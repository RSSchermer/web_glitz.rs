use std::borrow::Borrow;
use std::ops::Deref;
use std::sync::Arc;

use crate::buffer::BufferHandle;
use crate::sampler::{
    FloatSampler2DArrayHandle,
    FloatSampler2DHandle,
    FloatSampler3DHandle,
    FloatSamplerCubeHandle,
    IntegerSampler2DArrayHandle,
    IntegerSampler2DHandle,
    IntegerSampler3DHandle,
    IntegerSamplerCubeHandle,
    Sampler2DArrayShadowHandle,
    Sampler2DShadowHandle,
    SamplerCubeShadowHandle,
    UnsignedIntegerSampler2DArrayHandle,
    UnsignedIntegerSampler2DHandle,
    UnsignedIntegerSampler3DHandle,
    UnsignedIntegerSamplerCubeHandle,
};
use crate::std_140::Std140;

use super::{UniformIdentifier, IdentifierTail, IdentifierSegment};
use super::binding::BindingSlot;

pub trait Uniform {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError>;
}

pub enum BindingError {
    NotFound,
    TypeMismatch,
}

impl Uniform for f32 {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::Float(binder) = slot {
            binder.bind(*self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl Uniform for [f32] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        match slot {
            BindingSlot::ArrayOfFloat(binder) => {
                binder.bind(self);

                Ok(())
            }
            BindingSlot::FloatMatrix2x2(binder) if self.len() == 4 => {
                binder.bind([self[0], self[1], self[2], self[3]], false);

                Ok(())
            }
            BindingSlot::FloatMatrix2x3(binder) if self.len() == 6 => {
                binder.bind(
                    [self[0], self[1], self[2], self[3], self[4], self[5]],
                    false,
                );

                Ok(())
            }
            BindingSlot::FloatMatrix2x4(binder) if self.len() == 8 => {
                binder.bind(
                    [
                        self[0], self[1], self[2], self[3], self[4], self[5], self[6], self[7],
                    ],
                    false,
                );

                Ok(())
            }
            BindingSlot::FloatMatrix3x2(binder) if self.len() == 6 => {
                binder.bind(
                    [self[0], self[1], self[2], self[3], self[4], self[5]],
                    false,
                );

                Ok(())
            }
            BindingSlot::FloatMatrix3x3(binder) if self.len() == 9 => {
                binder.bind(
                    [
                        self[0], self[1], self[2], self[3], self[4], self[5], self[6], self[7],
                        self[8],
                    ],
                    false,
                );

                Ok(())
            }
            BindingSlot::FloatMatrix3x4(binder) if self.len() == 12 => {
                binder.bind(
                    [
                        self[0], self[1], self[2], self[3], self[4], self[5], self[6], self[7],
                        self[8], self[9], self[10], self[11],
                    ],
                    false,
                );

                Ok(())
            }
            BindingSlot::FloatMatrix4x2(binder) if self.len() == 8 => {
                binder.bind(
                    [
                        self[0], self[1], self[2], self[3], self[4], self[5], self[6], self[7],
                    ],
                    false,
                );

                Ok(())
            }
            BindingSlot::FloatMatrix4x3(binder) if self.len() == 12 => {
                binder.bind(
                    [
                        self[0], self[1], self[2], self[3], self[4], self[5], self[6], self[7],
                        self[8], self[9], self[10], self[11],
                    ],
                    false,
                );

                Ok(())
            }
            BindingSlot::FloatMatrix4x4(binder) if self.len() == 16 => {
                binder.bind(
                    [
                        self[0], self[1], self[2], self[3], self[4], self[5], self[6], self[7],
                        self[8], self[9], self[10], self[11], self[12], self[13], self[14],
                        self[15],
                    ],
                    false,
                );

                Ok(())
            }
            _ => Err(BindingError::TypeMismatch),
        }
    }
}

impl Uniform for (f32, f32) {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::FloatVector2(binder) = slot {
            binder.bind(*self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl Uniform for [(f32, f32)] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::ArrayOfFloatVector2(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl Uniform for (f32, f32, f32) {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::FloatVector3(binder) = slot {
            binder.bind(*self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl Uniform for [(f32, f32, f32)] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::ArrayOfFloatVector3(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl Uniform for (f32, f32, f32, f32) {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::FloatVector4(binder) = slot {
            binder.bind(*self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl Uniform for [(f32, f32, f32, f32)] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::ArrayOfFloatVector4(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl Uniform for [[f32; 4]] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::ArrayOfFloatMatrix2x2(binder) = slot {
            binder.bind(self, false);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl Uniform for [[f32; 6]] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        match slot {
            BindingSlot::ArrayOfFloatMatrix2x3(binder) => {
                binder.bind(self, false);

                Ok(())
            }
            BindingSlot::ArrayOfFloatMatrix3x2(binder) => {
                binder.bind(self, false);

                Ok(())
            }
            _ => Err(BindingError::TypeMismatch),
        }
    }
}

impl Uniform for [[f32; 8]] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        match slot {
            BindingSlot::ArrayOfFloatMatrix2x4(binder) => {
                binder.bind(self, false);

                Ok(())
            }
            BindingSlot::ArrayOfFloatMatrix4x2(binder) => {
                binder.bind(self, false);

                Ok(())
            }
            _ => Err(BindingError::TypeMismatch),
        }
    }
}

impl Uniform for [[f32; 9]] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::ArrayOfFloatMatrix3x3(binder) = slot {
            binder.bind(self, false);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl Uniform for [[f32; 12]] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        match slot {
            BindingSlot::ArrayOfFloatMatrix3x4(binder) => {
                binder.bind(self, false);

                Ok(())
            }
            BindingSlot::ArrayOfFloatMatrix4x3(binder) => {
                binder.bind(self, false);

                Ok(())
            }
            _ => Err(BindingError::TypeMismatch),
        }
    }
}

impl Uniform for [[f32; 16]] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::ArrayOfFloatMatrix4x4(binder) = slot {
            binder.bind(self, false);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl Uniform for i32 {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::Integer(binder) = slot {
            binder.bind(*self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl Uniform for [i32] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::ArrayOfInteger(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl Uniform for (i32, i32) {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::IntegerVector2(binder) = slot {
            binder.bind(*self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl Uniform for [(i32, i32)] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::ArrayOfIntegerVector2(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl Uniform for (i32, i32, i32) {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::IntegerVector3(binder) = slot {
            binder.bind(*self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl Uniform for [(i32, i32, i32)] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::ArrayOfIntegerVector3(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl Uniform for (i32, i32, i32, i32) {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::IntegerVector4(binder) = slot {
            binder.bind(*self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl Uniform for [(i32, i32, i32, i32)] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::ArrayOfIntegerVector4(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl Uniform for u32 {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        match slot {
            BindingSlot::UnsignedInteger(binder) => {
                binder.bind(*self);

                Ok(())
            }
            BindingSlot::Bool(binder) => {
                binder.bind(*self);

                Ok(())
            }
            _ => Err(BindingError::TypeMismatch),
        }
    }
}

impl Uniform for [u32] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        match slot {
            BindingSlot::ArrayOfUnsignedInteger(binder) => {
                binder.bind(self);

                Ok(())
            }
            BindingSlot::ArrayOfBool(binder) => {
                binder.bind(self);

                Ok(())
            }
            _ => Err(BindingError::TypeMismatch),
        }
    }
}

impl Uniform for (u32, u32) {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        match slot {
            BindingSlot::UnsignedIntegerVector2(binder) => {
                binder.bind(*self);

                Ok(())
            }
            BindingSlot::BoolVector2(binder) => {
                binder.bind(*self);

                Ok(())
            }
            _ => Err(BindingError::TypeMismatch),
        }
    }
}

impl Uniform for [(u32, u32)] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        match slot {
            BindingSlot::ArrayOfUnsignedIntegerVector2(binder) => {
                binder.bind(self);

                Ok(())
            }
            BindingSlot::ArrayOfBoolVector2(binder) => {
                binder.bind(self);

                Ok(())
            }
            _ => Err(BindingError::TypeMismatch),
        }
    }
}

impl Uniform for (u32, u32, u32) {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        match slot {
            BindingSlot::UnsignedIntegerVector3(binder) => {
                binder.bind(*self);

                Ok(())
            }
            BindingSlot::BoolVector3(binder) => {
                binder.bind(*self);

                Ok(())
            }
            _ => Err(BindingError::TypeMismatch),
        }
    }
}

impl Uniform for [(u32, u32, u32)] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        match slot {
            BindingSlot::ArrayOfUnsignedIntegerVector3(binder) => {
                binder.bind(self);

                Ok(())
            }
            BindingSlot::ArrayOfBoolVector3(binder) => {
                binder.bind(self);

                Ok(())
            }
            _ => Err(BindingError::TypeMismatch),
        }
    }
}

impl Uniform for (u32, u32, u32, u32) {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        match slot {
            BindingSlot::UnsignedIntegerVector4(binder) => {
                binder.bind(*self);

                Ok(())
            }
            BindingSlot::BoolVector4(binder) => {
                binder.bind(*self);

                Ok(())
            }
            _ => Err(BindingError::TypeMismatch),
        }
    }
}

impl Uniform for [(u32, u32, u32, u32)] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        match slot {
            BindingSlot::ArrayOfUnsignedIntegerVector4(binder) => {
                binder.bind(self);

                Ok(())
            }
            BindingSlot::ArrayOfBoolVector4(binder) => {
                binder.bind(self);

                Ok(())
            }
            _ => Err(BindingError::TypeMismatch),
        }
    }
}

impl Uniform for bool {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::Bool(binder) = slot {
            binder.bind((*self).into());

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl Uniform for [bool] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::ArrayOfBool(binder) = slot {
            let value: Vec<u32> = self.into_iter().map(|v| (*v).into()).collect();

            binder.bind(&value);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl Uniform for (bool, bool) {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::BoolVector2(binder) = slot {
            let (v0, v1) = *self;

            binder.bind((v0.into(), v1.into()));

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl Uniform for [(bool, bool)] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::ArrayOfBoolVector2(binder) = slot {
            let value: Vec<(u32, u32)> = self
                .into_iter()
                .map(|v| {
                    let (v0, v1) = *v;

                    (v0.into(), v1.into())
                })
                .collect();

            binder.bind(&value);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl Uniform for (bool, bool, bool) {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::BoolVector3(binder) = slot {
            let (v0, v1, v2) = *self;

            binder.bind((v0.into(), v1.into(), v2.into()));

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl Uniform for [(bool, bool, bool)] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::ArrayOfBoolVector3(binder) = slot {
            let value: Vec<(u32, u32, u32)> = self
                .into_iter()
                .map(|v| {
                    let (v0, v1, v2) = *v;

                    (v0.into(), v1.into(), v2.into())
                })
                .collect();

            binder.bind(&value);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl Uniform for (bool, bool, bool, bool) {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::BoolVector4(binder) = slot {
            let (v0, v1, v2, v3) = *self;

            binder.bind((v0.into(), v1.into(), v2.into(), v3.into()));

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl Uniform for [(bool, bool, bool, bool)] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::ArrayOfBoolVector4(binder) = slot {
            let value: Vec<(u32, u32, u32, u32)> = self
                .into_iter()
                .map(|v| {
                    let (v0, v1, v2, v3) = *v;

                    (v0.into(), v1.into(), v2.into(), v3.into())
                })
                .collect();

            binder.bind(&value);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<F> Uniform for FloatSampler2DHandle<F> {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::FloatSampler2D(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<'a, F> Uniform for &'a [FloatSampler2DHandle<F>] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::ArrayOfFloatSampler2D(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<F> Uniform for IntegerSampler2DHandle<F> {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::IntegerSampler2D(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<'a, F> Uniform for &'a [IntegerSampler2DHandle<F>] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::ArrayOfIntegerSampler2D(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<F> Uniform for UnsignedIntegerSampler2DHandle<F> {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::UnsignedIntegerSampler2D(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<'a, F> Uniform for &'a [UnsignedIntegerSampler2DHandle<F>] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::ArrayOfUnsignedIntegerSampler2D(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<F> Uniform for FloatSampler2DArrayHandle<F> {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::FloatSampler2DArray(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<'a, F> Uniform for &'a [FloatSampler2DArrayHandle<F>] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::ArrayOfFloatSampler2DArray(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<F> Uniform for IntegerSampler2DArrayHandle<F> {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::IntegerSampler2DArray(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<'a, F> Uniform for &'a [IntegerSampler2DArrayHandle<F>] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::ArrayOfIntegerSampler2DArray(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<F> Uniform for UnsignedIntegerSampler2DArrayHandle<F> {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::UnsignedIntegerSampler2DArray(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<'a, F> Uniform for &'a [UnsignedIntegerSampler2DArrayHandle<F>] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::ArrayOfUnsignedIntegerSampler2DArray(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<F> Uniform for FloatSampler3DHandle<F> {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::FloatSampler3D(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<'a, F> Uniform for &'a [FloatSampler3DHandle<F>] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::ArrayOfFloatSampler3D(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<F> Uniform for IntegerSampler3DHandle<F> {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::IntegerSampler3D(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<'a, F> Uniform for &'a [IntegerSampler3DHandle<F>] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::ArrayOfIntegerSampler3D(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<F> Uniform for UnsignedIntegerSampler3DHandle<F> {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::UnsignedIntegerSampler3D(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<'a, F> Uniform for &'a [UnsignedIntegerSampler3DHandle<F>] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::ArrayOfUnsignedIntegerSampler3D(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<F> Uniform for FloatSamplerCubeHandle<F> {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::FloatSamplerCube(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<'a, F> Uniform for &'a [FloatSamplerCubeHandle<F>] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::ArrayOfFloatSamplerCube(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<F> Uniform for IntegerSamplerCubeHandle<F> {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::IntegerSamplerCube(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<'a, F> Uniform for &'a [IntegerSamplerCubeHandle<F>] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::ArrayOfIntegerSamplerCube(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<F> Uniform for UnsignedIntegerSamplerCubeHandle<F> {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::UnsignedIntegerSamplerCube(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<'a, F> Uniform for &'a [UnsignedIntegerSamplerCubeHandle<F>] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::ArrayOfUnsignedIntegerSamplerCube(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<F> Uniform for Sampler2DShadowHandle<F> {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::Sampler2DShadow(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<'a, F> Uniform for &'a [Sampler2DShadowHandle<F>] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::ArrayOfSampler2DShadow(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<F> Uniform for Sampler2DArrayShadowHandle<F> {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::Sampler2DArrayShadow(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<'a, F> Uniform for &'a [Sampler2DArrayShadowHandle<F>] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::ArrayOfSampler2DArrayShadow(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<F> Uniform for SamplerCubeShadowHandle<F> {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::SamplerCubeShadow(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<'a, F> Uniform for &'a [SamplerCubeShadowHandle<F>] {
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_array_terminus() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::ArrayOfSamplerCubeShadow(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<T> Uniform for BufferHandle<T>
where
    T: Std140,
{
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        if !identifier.is_empty() {
            return Err(BindingError::NotFound);
        }

        if let BindingSlot::Block(binder) = slot {
            binder.bind(self);

            Ok(())
        } else {
            Err(BindingError::TypeMismatch)
        }
    }
}

impl<T> Uniform for Vec<T>
where
    [T]: Uniform,
{
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        self.deref().bind(identifier, slot)
    }
}

impl<T> Uniform for Box<T>
where
    T: Uniform,
{
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        self.deref().bind(identifier, slot)
    }
}

impl<T> Uniform for Box<[T]>
where
    [T]: Uniform,
{
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        self.deref().bind(identifier, slot)
    }
}

impl<T> Uniform for Arc<T>
where
    T: Uniform,
{
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        self.deref().bind(identifier, slot)
    }
}

impl<T> Uniform for Arc<[T]>
where
    [T]: Uniform,
{
    fn bind(&self, identifier: IdentifierTail, slot: &mut BindingSlot) -> Result<(), BindingError> {
        self.deref().bind(identifier, slot)
    }
}

macro_rules! generate_array_impl {
    ($size:tt) => {
        impl<T> Uniform for [T; $size]
        where
            [T]: Uniform,
        {
            fn bind(
                &self,
                identifier: IdentifierTail,
                slot: &mut BindingSlot,
            ) -> Result<(), BindingError> {
                <[T] as Uniform>::bind(self.as_ref(), identifier, slot)
            }
        }
    };
}

generate_array_impl!(0);
generate_array_impl!(1);
generate_array_impl!(2);
generate_array_impl!(3);
generate_array_impl!(4);
generate_array_impl!(5);
generate_array_impl!(6);
generate_array_impl!(7);
generate_array_impl!(8);
generate_array_impl!(9);
generate_array_impl!(10);
generate_array_impl!(11);
generate_array_impl!(12);
generate_array_impl!(13);
generate_array_impl!(14);
generate_array_impl!(15);
generate_array_impl!(16);
generate_array_impl!(17);
generate_array_impl!(18);
generate_array_impl!(19);
generate_array_impl!(20);
generate_array_impl!(21);
generate_array_impl!(22);
generate_array_impl!(23);
generate_array_impl!(24);
generate_array_impl!(25);
generate_array_impl!(26);
generate_array_impl!(27);
generate_array_impl!(28);
generate_array_impl!(29);
generate_array_impl!(30);
generate_array_impl!(31);
generate_array_impl!(32);
