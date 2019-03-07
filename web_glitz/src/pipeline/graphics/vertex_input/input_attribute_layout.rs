use super::Vertex;

pub unsafe trait InputAttributeLayout {
    fn check_compatibility(slot_descriptors: &[AttributeSlotDescriptor]) -> Result<(), Incompatible>;
}

pub enum Incompatible {
    MissingAttribute {
        location: u32
    },
    TypeMismatch {
        location: u32
    }
}

pub struct AttributeSlotDescriptor {
    location: u32,
    attribute_type: AttributeType
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
