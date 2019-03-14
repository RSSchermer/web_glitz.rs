use super::Vertex;

pub unsafe trait InputAttributeLayout {
    fn check_compatibility(
        slot_descriptors: &[AttributeSlotDescriptor],
    ) -> Result<(), Incompatible>;
}

pub enum Incompatible {
    MissingAttribute { location: u32 },
    TypeMismatch { location: u32 },
}

pub struct AttributeSlotDescriptor {
    location: u32,
    attribute_type: AttributeType,
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
            Gl::FLOAT_MAT2x3 => AttributeType::FloatMatrix2x3,
            Gl::FLOAT_MAT2x4 => AttributeType::FloatMatrix2x4,
            Gl::FLOAT_MAT3x2 => AttributeType::FloatMatrix3x2,
            Gl::FLOAT_MAT3x4 => AttributeType::FloatMatrix3x4,
            Gl::FLOAT_MAT4x2 => AttributeType::FloatMatrix4x2,
            Gl::FLOAT_MAT4x3 => AttributeType::FloatMatrix4x3,
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

macro_rules! impl_input_attribute_layout {
    ($($T:ident),*) => {
        unsafe impl<$($T),*> InputAttributeLayout for ($($T),*) where $($T: Vertex),* {
            fn check_compatibility(slot_descriptors: &[AttributeSlotDescriptor]) -> Result<(), Incompatible> {
                'outer: for slot in slot_descriptors.iter() {
                    $(
                        'inner: for attribute in $T::input_attribute_descriptors().iter() {
                            if attribute.location == slot.location() {
                                if !attribute.format.is_compatible(slot.attribute_type) {
                                    return Err(Incompatible::TypeMismatch { location: slot.location() })
                                }

                                continue 'outer;
                            }
                        }
                    )*

                    return Err(Incompatible::MissingAttribute { location: slot.location() })
                }

                Ok(())
            }
        }
    }
}

impl_input_attribute_layout!(T0);
impl_input_attribute_layout!(T0, T1);
impl_input_attribute_layout!(T0, T1, T2);
impl_input_attribute_layout!(T0, T1, T2, T3);
impl_input_attribute_layout!(T0, T1, T2, T3, T4);
impl_input_attribute_layout!(T0, T1, T2, T3, T4, T5);
impl_input_attribute_layout!(T0, T1, T2, T3, T4, T5, T6);
impl_input_attribute_layout!(T0, T1, T2, T3, T4, T5, T6, T7);
impl_input_attribute_layout!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
impl_input_attribute_layout!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_input_attribute_layout!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_input_attribute_layout!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_input_attribute_layout!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_input_attribute_layout!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_input_attribute_layout!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_input_attribute_layout!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
