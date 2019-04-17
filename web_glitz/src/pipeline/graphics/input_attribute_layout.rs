use std::borrow::Borrow;

use web_sys::WebGl2RenderingContext as Gl;

use crate::vertex::VertexAttributeLayout;

pub unsafe trait AttributeSlotLayoutCompatible {
    fn check_compatibility(
        slot_descriptors: &[AttributeSlotDescriptor],
    ) -> Result<(), IncompatibleAttributeLayout>;
}

#[derive(Debug)]
pub enum IncompatibleAttributeLayout {
    MissingAttribute { location: u32 },
    TypeMismatch { location: u32 },
}

pub struct AttributeSlotDescriptor {
    pub(crate) location: u32,
    pub(crate) attribute_type: AttributeType,
}

impl AttributeSlotDescriptor {
    pub fn location(&self) -> u32 {
        self.location
    }

    pub fn attribute_type(&self) -> AttributeType {
        self.attribute_type
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AttributeType {
    Float,
    FloatVector2,
    FloatVector3,
    FloatVector4,
    FloatMatrix2x2,
    FloatMatrix2x3,
    FloatMatrix2x4,
    FloatMatrix3x2,
    FloatMatrix3x3,
    FloatMatrix3x4,
    FloatMatrix4x2,
    FloatMatrix4x3,
    FloatMatrix4x4,
    Integer,
    IntegerVector2,
    IntegerVector3,
    IntegerVector4,
    UnsignedInteger,
    UnsignedIntegerVector2,
    UnsignedIntegerVector3,
    UnsignedIntegerVector4,
}

impl AttributeType {
    pub(crate) fn from_type_id(id: u32) -> Self {
        match id {
            Gl::FLOAT => AttributeType::Float,
            Gl::FLOAT_VEC2 => AttributeType::FloatVector2,
            Gl::FLOAT_VEC3 => AttributeType::FloatVector3,
            Gl::FLOAT_VEC4 => AttributeType::FloatVector4,
            Gl::FLOAT_MAT2 => AttributeType::FloatMatrix2x2,
            Gl::FLOAT_MAT3 => AttributeType::FloatMatrix3x3,
            Gl::FLOAT_MAT4 => AttributeType::FloatMatrix4x4,
            Gl::FLOAT_MAT2X3 => AttributeType::FloatMatrix2x3,
            Gl::FLOAT_MAT2X4 => AttributeType::FloatMatrix2x4,
            Gl::FLOAT_MAT3X2 => AttributeType::FloatMatrix3x2,
            Gl::FLOAT_MAT3X4 => AttributeType::FloatMatrix3x4,
            Gl::FLOAT_MAT4X2 => AttributeType::FloatMatrix4x2,
            Gl::FLOAT_MAT4X3 => AttributeType::FloatMatrix4x3,
            Gl::INT => AttributeType::Integer,
            Gl::INT_VEC2 => AttributeType::IntegerVector2,
            Gl::INT_VEC3 => AttributeType::IntegerVector3,
            Gl::INT_VEC4 => AttributeType::IntegerVector4,
            Gl::UNSIGNED_INT => AttributeType::UnsignedInteger,
            Gl::UNSIGNED_INT_VEC2 => AttributeType::UnsignedIntegerVector2,
            Gl::UNSIGNED_INT_VEC3 => AttributeType::UnsignedIntegerVector3,
            Gl::UNSIGNED_INT_VEC4 => AttributeType::UnsignedIntegerVector4,
            id => panic!("Invalid attribute type id: {}", id),
        }
    }
}

unsafe impl<T> AttributeSlotLayoutCompatible for T where T: VertexAttributeLayout {
    fn check_compatibility(
        slot_descriptors: &[AttributeSlotDescriptor],
    ) -> Result<(), IncompatibleAttributeLayout> {
        'outer: for slot in slot_descriptors.iter() {
            for bind_group in T::input_attribute_bindings().borrow() {
                for attribute_descriptor in bind_group.iter() {
                    if attribute_descriptor.location == slot.location() {
                        if !attribute_descriptor.format.is_compatible(slot.attribute_type) {
                            return Err(IncompatibleAttributeLayout::TypeMismatch { location: slot.location() })
                        }

                        continue 'outer;
                    }
                }
            }

            return Err(IncompatibleAttributeLayout::MissingAttribute { location: slot.location() })
        }

        Ok(())
    }
}
