use std::borrow::Borrow;

use web_sys::WebGl2RenderingContext as Gl;

use crate::vertex::VertexAttributeLayout;

/// Trait implemented by vertex types that may be compatible with a [GraphicsPipeline]'s attribute
/// slot layout.
///
/// A [GraphicsPipeline] may specify its vertex type to be an [AttributeSlotLayoutCompatible] type
/// (see, [GraphicsPipelineDescriptorBuilder::vertex_input_layout]). If it does, then when the
/// graphics pipeline is being created (see [RenderingContext::create_graphics_pipeline]),
/// [check_compatibility] will be called with the actual set of [AttributeSlotDescriptor]s defined
/// by the pipeline's programmable shaders (as obtained by reflection on the shader source code). If
/// [check_compatibility] returns an error, then [RenderingContext::create_graphics_pipeline] will
/// fail to create the pipeline and will return an error; if [check_compatibility] does not return
/// an error, then any [VertexStreamDescription] that uses the type as its vertex type may be safely
/// used as the vertex stream for a draw command that uses the pipeline (see
/// [PipelineTask::draw_command]), without additional runtime compatibility checks.
///
/// # Unsafe
///
/// If the implementation of [check_compatibility] for some type does not return an error when
/// called with a certain set of [AttributeSlotDescriptor]s, then any safely constructed
/// [VertexArray] that uses that type as its vertex type must provide attribute data that is
/// compatible with that set of [AttributeSlotDescriptor]s.
pub unsafe trait AttributeSlotLayoutCompatible {
    /// Returns `Ok` if this type is compatible with the pipeline attribute slot layout described by
    /// `slot_descriptors`, or an returns an [IncompatibleAttributeLayout] otherwise.
    fn check_compatibility(
        slot_descriptors: &[AttributeSlotDescriptor],
    ) -> Result<(), IncompatibleAttributeLayout>;
}

/// Error returned by [AttributeSlotLayoutCompatible::check_compatibility].
#[derive(Debug)]
pub enum IncompatibleAttributeLayout {
    /// Variant returned if no attribute data is available for the [AttributeSlotDescriptor] with
    /// at the `location`.
    MissingAttribute { location: u32 },

    /// Variant returned if attribute data is available for the [AttributeSlotDescriptor] with
    /// at the `location`. but attribute data is not compatible with the [AttributeType] of the
    /// [AttributeSlotDescriptor] (see [AttributeSlotDescriptor::attribute_type]).
    TypeMismatch { location: u32 },
}

/// Describes an input slot on a [GraphicsPipeline].
pub struct AttributeSlotDescriptor {
    pub(crate) location: u32,
    pub(crate) attribute_type: AttributeType,
}

impl AttributeSlotDescriptor {
    /// The shader location of the attribute slot.
    pub fn location(&self) -> u32 {
        self.location
    }

    /// The type of attribute required to fill the slot.
    pub fn attribute_type(&self) -> AttributeType {
        self.attribute_type
    }
}

/// Enumerates the possible attribute types that might be required to fill an attribute slot.
///
/// See also [AttributeSlotDescriptor].
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

unsafe impl<T> AttributeSlotLayoutCompatible for T
where
    T: VertexAttributeLayout,
{
    fn check_compatibility(
        slot_descriptors: &[AttributeSlotDescriptor],
    ) -> Result<(), IncompatibleAttributeLayout> {
        'outer: for slot in slot_descriptors.iter() {
            for bind_group in T::input_attribute_bindings().borrow() {
                for attribute_descriptor in bind_group.iter() {
                    if attribute_descriptor.location == slot.location() {
                        if !attribute_descriptor
                            .format
                            .is_compatible(slot.attribute_type)
                        {
                            return Err(IncompatibleAttributeLayout::TypeMismatch {
                                location: slot.location(),
                            });
                        }

                        continue 'outer;
                    }
                }
            }

            return Err(IncompatibleAttributeLayout::MissingAttribute {
                location: slot.location(),
            });
        }

        Ok(())
    }
}
