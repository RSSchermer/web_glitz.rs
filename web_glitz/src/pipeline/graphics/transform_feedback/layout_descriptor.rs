use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};

use web_sys::{WebGl2RenderingContext as Gl, WebGlProgram};

use crate::runtime::CreateGraphicsPipelineError;

pub unsafe trait TransformFeedback {
    const ATTRIBUTE_DESCRIPTORS: &'static [TransformFeedbackAttributeDescriptor];
}

/// A transform feedback layout description attached to a type.
///
/// See also [TransformFeedbackAttributeDescriptor].
///
/// This trait becomes useful in combination with the [TypedTransformFeedbackBuffers] trait. If a
/// [TypedTransformFeedbackLayout] is attached to a [GraphicsPipeline] (see
/// [GraphicsPipelineDescriptorBuilder::typed_transform_layout]), then
/// [TypedTransformFeedbackBuffers] with a matching [TypedTransformFeedbackBuffers::Layout] may be
/// bound to the pipeline without further runtime checks.
///
/// Note that [TypedTransformFeedbackLayout] is safe to implement, but
/// [TypedTransformFeedbackBuffers] is unsafe: the data in the buffer representation for which
/// [TypedTransformFeedbackBuffers] is  implemented must always be compatible with the transform
/// feedback layout specified by the [TypedTransformFeedbackBuffers::Layout], see
/// [TypedTransformFeedbackBuffers] for details.
pub unsafe trait TypedTransformFeedbackLayout {
    type LayoutDescription: Into<TransformFeedbackLayoutDescriptor>;

    const LAYOUT_DESCRIPTION: Self::LayoutDescription;
}

macro_rules! impl_typed_transform_feedback_layout {
    ($n:tt, $($T:ident),*) => {
        unsafe impl<$($T),*> TypedTransformFeedbackLayout for ($($T),*)
        where
            $($T: TransformFeedback),*
        {
            type LayoutDescription = [&'static [TransformFeedbackAttributeDescriptor]; $n];

            const LAYOUT_DESCRIPTION: Self::LayoutDescription = [
                $($T::ATTRIBUTE_DESCRIPTORS),*
            ];
        }
    }
}

impl_typed_transform_feedback_layout!(1, T0);
impl_typed_transform_feedback_layout!(2, T0, T1);
impl_typed_transform_feedback_layout!(3, T0, T1, T2);
impl_typed_transform_feedback_layout!(4, T0, T1, T2, T3);
impl_typed_transform_feedback_layout!(5, T0, T1, T2, T3, T4);
impl_typed_transform_feedback_layout!(6, T0, T1, T2, T3, T4, T5);
impl_typed_transform_feedback_layout!(7, T0, T1, T2, T3, T4, T5, T6);
impl_typed_transform_feedback_layout!(8, T0, T1, T2, T3, T4, T5, T6, T7);
impl_typed_transform_feedback_layout!(9, T0, T1, T2, T3, T4, T5, T6, T7, T8);
impl_typed_transform_feedback_layout!(10, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_typed_transform_feedback_layout!(11, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_typed_transform_feedback_layout!(12, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_typed_transform_feedback_layout!(13, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_typed_transform_feedback_layout!(
    14, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13
);
impl_typed_transform_feedback_layout!(
    15, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14
);
impl_typed_transform_feedback_layout!(
    16, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15
);

impl Into<TransformFeedbackLayoutDescriptor> for () {
    fn into(self) -> TransformFeedbackLayoutDescriptor {
        TransformFeedbackLayoutDescriptor {
            layout: Vec::new(), // Won't allocate, see [Vec::new].
        }
    }
}

macro_rules! impl_into_transform_feedback_layout_descriptor {
    ($n:tt) => {
        impl Into<TransformFeedbackLayoutDescriptor>
            for [&'static [TransformFeedbackAttributeDescriptor]; $n]
        {
            fn into(self) -> TransformFeedbackLayoutDescriptor {
                let mut attribute_count = 0;

                for i in 0..$n {
                    attribute_count += self[i].len();
                }

                let mut builder = TransformFeedbackLayoutDescriptorBuilder::new(Some(
                    TransformFeedbackLayoutAllocationHint {
                        bind_slot_count: $n,
                        attribute_count: attribute_count as u8,
                    },
                ));

                for i in 0..$n {
                    let mut slot = builder.add_buffer_slot();

                    for attribute in self[i] {
                        slot.add_attribute(attribute.clone());
                    }
                }

                builder.finish()
            }
        }
    };
}

impl_into_transform_feedback_layout_descriptor!(1);
impl_into_transform_feedback_layout_descriptor!(2);
impl_into_transform_feedback_layout_descriptor!(3);
impl_into_transform_feedback_layout_descriptor!(4);
impl_into_transform_feedback_layout_descriptor!(5);
impl_into_transform_feedback_layout_descriptor!(6);
impl_into_transform_feedback_layout_descriptor!(7);
impl_into_transform_feedback_layout_descriptor!(8);
impl_into_transform_feedback_layout_descriptor!(9);
impl_into_transform_feedback_layout_descriptor!(10);
impl_into_transform_feedback_layout_descriptor!(11);
impl_into_transform_feedback_layout_descriptor!(12);
impl_into_transform_feedback_layout_descriptor!(13);
impl_into_transform_feedback_layout_descriptor!(14);
impl_into_transform_feedback_layout_descriptor!(15);
impl_into_transform_feedback_layout_descriptor!(16);

/// Describes how the data from a transform stage output attribute is recorded into transform
/// feedback buffers.
///
/// See also [TransformFeedbackLayoutDescriptor].
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct TransformFeedbackAttributeDescriptor {
    /// The name of the output attribute as defined by the shader code.
    pub name: String,

    /// The type of the output attribute.
    pub attribute_type: TransformFeedbackAttributeType,

    /// The array size (length) of the output attribute.
    ///
    /// Always `1` for non-array attributes.
    pub size: usize,
}

/// Enumerates the possible transform stage output types that may be recorded as transform feedback.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TransformFeedbackAttributeType {
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

impl TransformFeedbackAttributeType {
    pub(crate) fn from_type_id(id: u32) -> Self {
        match id {
            Gl::FLOAT => TransformFeedbackAttributeType::Float,
            Gl::FLOAT_VEC2 => TransformFeedbackAttributeType::FloatVector2,
            Gl::FLOAT_VEC3 => TransformFeedbackAttributeType::FloatVector3,
            Gl::FLOAT_VEC4 => TransformFeedbackAttributeType::FloatVector4,
            Gl::FLOAT_MAT2 => TransformFeedbackAttributeType::FloatMatrix2x2,
            Gl::FLOAT_MAT3 => TransformFeedbackAttributeType::FloatMatrix3x3,
            Gl::FLOAT_MAT4 => TransformFeedbackAttributeType::FloatMatrix4x4,
            Gl::FLOAT_MAT2X3 => TransformFeedbackAttributeType::FloatMatrix2x3,
            Gl::FLOAT_MAT2X4 => TransformFeedbackAttributeType::FloatMatrix2x4,
            Gl::FLOAT_MAT3X2 => TransformFeedbackAttributeType::FloatMatrix3x2,
            Gl::FLOAT_MAT3X4 => TransformFeedbackAttributeType::FloatMatrix3x4,
            Gl::FLOAT_MAT4X2 => TransformFeedbackAttributeType::FloatMatrix4x2,
            Gl::FLOAT_MAT4X3 => TransformFeedbackAttributeType::FloatMatrix4x3,
            Gl::INT => TransformFeedbackAttributeType::Integer,
            Gl::INT_VEC2 => TransformFeedbackAttributeType::IntegerVector2,
            Gl::INT_VEC3 => TransformFeedbackAttributeType::IntegerVector3,
            Gl::INT_VEC4 => TransformFeedbackAttributeType::IntegerVector4,
            Gl::UNSIGNED_INT => TransformFeedbackAttributeType::UnsignedInteger,
            Gl::UNSIGNED_INT_VEC2 => TransformFeedbackAttributeType::UnsignedIntegerVector2,
            Gl::UNSIGNED_INT_VEC3 => TransformFeedbackAttributeType::UnsignedIntegerVector3,
            Gl::UNSIGNED_INT_VEC4 => TransformFeedbackAttributeType::UnsignedIntegerVector4,
            id => panic!("Invalid feedback varying type id: {}", id),
        }
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct TransformFeedbackLayoutDescriptor {
    layout: Vec<LayoutElement>,
}

impl TransformFeedbackLayoutDescriptor {
    pub(crate) fn check_compatibility(
        &self,
        program: &WebGlProgram,
        gl: &Gl,
    ) -> Result<(), CreateGraphicsPipelineError> {
        let mut index = 0;

        for group in self.buffer_slots() {
            for attribute in group.attributes() {
                let info = gl.get_transform_feedback_varying(program, index).unwrap();

                if info.size() != attribute.size as i32 {
                    return Err(CreateGraphicsPipelineError::TransformFeedbackTypeMismatch(
                        attribute.name.clone(),
                    ));
                }

                match attribute.attribute_type {
                    TransformFeedbackAttributeType::Float => {
                        if info.type_() != Gl::FLOAT {
                            return Err(
                                CreateGraphicsPipelineError::TransformFeedbackTypeMismatch(
                                    attribute.name.clone(),
                                ),
                            );
                        }
                    }
                    TransformFeedbackAttributeType::FloatVector2 => {
                        if info.type_() != Gl::FLOAT_VEC2 {
                            return Err(
                                CreateGraphicsPipelineError::TransformFeedbackTypeMismatch(
                                    attribute.name.clone(),
                                ),
                            );
                        }
                    }
                    TransformFeedbackAttributeType::FloatVector3 => {
                        if info.type_() != Gl::FLOAT_VEC3 {
                            return Err(
                                CreateGraphicsPipelineError::TransformFeedbackTypeMismatch(
                                    attribute.name.clone(),
                                ),
                            );
                        }
                    }
                    TransformFeedbackAttributeType::FloatVector4 => {
                        if info.type_() != Gl::FLOAT_VEC4 {
                            return Err(
                                CreateGraphicsPipelineError::TransformFeedbackTypeMismatch(
                                    attribute.name.clone(),
                                ),
                            );
                        }
                    }
                    TransformFeedbackAttributeType::FloatMatrix2x2 => {
                        if info.type_() != Gl::FLOAT_MAT2 {
                            return Err(
                                CreateGraphicsPipelineError::TransformFeedbackTypeMismatch(
                                    attribute.name.clone(),
                                ),
                            );
                        }
                    }
                    TransformFeedbackAttributeType::FloatMatrix2x3 => {
                        if info.type_() != Gl::FLOAT_MAT2X3 {
                            return Err(
                                CreateGraphicsPipelineError::TransformFeedbackTypeMismatch(
                                    attribute.name.clone(),
                                ),
                            );
                        }
                    }
                    TransformFeedbackAttributeType::FloatMatrix2x4 => {
                        if info.type_() != Gl::FLOAT_MAT2X4 {
                            return Err(
                                CreateGraphicsPipelineError::TransformFeedbackTypeMismatch(
                                    attribute.name.clone(),
                                ),
                            );
                        }
                    }
                    TransformFeedbackAttributeType::FloatMatrix3x2 => {
                        if info.type_() != Gl::FLOAT_MAT3X2 {
                            return Err(
                                CreateGraphicsPipelineError::TransformFeedbackTypeMismatch(
                                    attribute.name.clone(),
                                ),
                            );
                        }
                    }
                    TransformFeedbackAttributeType::FloatMatrix3x3 => {
                        if info.type_() != Gl::FLOAT_MAT3 {
                            return Err(
                                CreateGraphicsPipelineError::TransformFeedbackTypeMismatch(
                                    attribute.name.clone(),
                                ),
                            );
                        }
                    }
                    TransformFeedbackAttributeType::FloatMatrix3x4 => {
                        if info.type_() != Gl::FLOAT_MAT3X4 {
                            return Err(
                                CreateGraphicsPipelineError::TransformFeedbackTypeMismatch(
                                    attribute.name.clone(),
                                ),
                            );
                        }
                    }
                    TransformFeedbackAttributeType::FloatMatrix4x2 => {
                        if info.type_() != Gl::FLOAT_MAT4X2 {
                            return Err(
                                CreateGraphicsPipelineError::TransformFeedbackTypeMismatch(
                                    attribute.name.clone(),
                                ),
                            );
                        }
                    }
                    TransformFeedbackAttributeType::FloatMatrix4x3 => {
                        if info.type_() != Gl::FLOAT_MAT4X3 {
                            return Err(
                                CreateGraphicsPipelineError::TransformFeedbackTypeMismatch(
                                    attribute.name.clone(),
                                ),
                            );
                        }
                    }
                    TransformFeedbackAttributeType::FloatMatrix4x4 => {
                        if info.type_() != Gl::FLOAT_MAT4 {
                            return Err(
                                CreateGraphicsPipelineError::TransformFeedbackTypeMismatch(
                                    attribute.name.clone(),
                                ),
                            );
                        }
                    }
                    TransformFeedbackAttributeType::Integer => {
                        if info.type_() != Gl::INT {
                            return Err(
                                CreateGraphicsPipelineError::TransformFeedbackTypeMismatch(
                                    attribute.name.clone(),
                                ),
                            );
                        }
                    }
                    TransformFeedbackAttributeType::IntegerVector2 => {
                        if info.type_() != Gl::INT_VEC2 {
                            return Err(
                                CreateGraphicsPipelineError::TransformFeedbackTypeMismatch(
                                    attribute.name.clone(),
                                ),
                            );
                        }
                    }
                    TransformFeedbackAttributeType::IntegerVector3 => {
                        if info.type_() != Gl::INT_VEC3 {
                            return Err(
                                CreateGraphicsPipelineError::TransformFeedbackTypeMismatch(
                                    attribute.name.clone(),
                                ),
                            );
                        }
                    }
                    TransformFeedbackAttributeType::IntegerVector4 => {
                        if info.type_() != Gl::INT_VEC4 {
                            return Err(
                                CreateGraphicsPipelineError::TransformFeedbackTypeMismatch(
                                    attribute.name.clone(),
                                ),
                            );
                        }
                    }
                    TransformFeedbackAttributeType::UnsignedInteger => {
                        if info.type_() != Gl::UNSIGNED_INT {
                            return Err(
                                CreateGraphicsPipelineError::TransformFeedbackTypeMismatch(
                                    attribute.name.clone(),
                                ),
                            );
                        }
                    }
                    TransformFeedbackAttributeType::UnsignedIntegerVector2 => {
                        if info.type_() != Gl::UNSIGNED_INT_VEC2 {
                            return Err(
                                CreateGraphicsPipelineError::TransformFeedbackTypeMismatch(
                                    attribute.name.clone(),
                                ),
                            );
                        }
                    }
                    TransformFeedbackAttributeType::UnsignedIntegerVector3 => {
                        if info.type_() != Gl::UNSIGNED_INT_VEC3 {
                            return Err(
                                CreateGraphicsPipelineError::TransformFeedbackTypeMismatch(
                                    attribute.name.clone(),
                                ),
                            );
                        }
                    }
                    TransformFeedbackAttributeType::UnsignedIntegerVector4 => {
                        if info.type_() != Gl::UNSIGNED_INT_VEC4 {
                            return Err(
                                CreateGraphicsPipelineError::TransformFeedbackTypeMismatch(
                                    attribute.name.clone(),
                                ),
                            );
                        }
                    }
                }

                index += 1;
            }
        }

        Ok(())
    }

    /// Returns an iterator over the transform feedback buffer binding slots described by this
    /// descriptor.
    pub fn buffer_slots(&self) -> TransformFeedbackBufferSlots {
        TransformFeedbackBufferSlots {
            layout: self,
            cursor: -1,
        }
    }
}

/// Returned from [TransformFeedbackLayoutDescriptor::buffer_slots].
pub struct TransformFeedbackBufferSlots<'a> {
    layout: &'a TransformFeedbackLayoutDescriptor,
    cursor: isize,
}

impl<'a> Iterator for TransformFeedbackBufferSlots<'a> {
    type Item = TransformFeedbackBufferSlotRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor < 0 {
            self.cursor += 1;

            Some(TransformFeedbackBufferSlotRef {
                layout: self.layout,
                start: 0,
            })
        } else {
            while let Some(element) = self.layout.layout.get(self.cursor as usize) {
                self.cursor += 1;

                if let LayoutElement::NextBindSlot = element {
                    return Some(TransformFeedbackBufferSlotRef {
                        layout: self.layout,
                        start: self.cursor as usize,
                    });
                }
            }

            None
        }
    }
}

/// Reference to a bind slot description in a [TransformFeedbackLayoutDescriptor].
///
/// See [TransformFeedbackLayoutDescriptor::feedback_buffer_slots].
pub struct TransformFeedbackBufferSlotRef<'a> {
    layout: &'a TransformFeedbackLayoutDescriptor,
    start: usize,
}

impl<'a> TransformFeedbackBufferSlotRef<'a> {
    /// Returns an iterator over the [FeedbackAttributeDescriptor]s defined on this bind slot.
    pub fn attributes(&self) -> TransformFeedbackBufferSlotAttributes {
        TransformFeedbackBufferSlotAttributes {
            layout: &self.layout.layout,
            cursor: self.start,
        }
    }
}

/// Returned from [TransformFeedbackBufferSlotRef::attributes].
pub struct TransformFeedbackBufferSlotAttributes<'a> {
    layout: &'a Vec<LayoutElement>,
    cursor: usize,
}

impl<'a> Iterator for TransformFeedbackBufferSlotAttributes<'a> {
    type Item = &'a TransformFeedbackAttributeDescriptor;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(LayoutElement::NextAttribute(attribute)) = self.layout.get(self.cursor) {
            self.cursor += 1;

            Some(attribute)
        } else {
            None
        }
    }
}

#[derive(Clone, PartialEq, Hash, Eq, Debug)]
enum LayoutElement {
    NextAttribute(TransformFeedbackAttributeDescriptor),
    NextBindSlot,
}

/// Allocation hint for a [TransformFeedbackLayoutDescriptor], see
/// [TransformFeedbackLayoutDescriptorBuilder::new].
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct TransformFeedbackLayoutAllocationHint {
    /// The number of bind slots the descriptor describes.
    pub bind_slot_count: u8,

    /// The total number of attributes the descriptor describes across all bind slots.
    pub attribute_count: u8,
}

/// Builds a new [TransformFeedbackLayoutDescriptor].
///
/// # Example
///
/// ```rust
/// use web_glitz::pipeline::graphics::{TransformFeedbackLayoutAllocationHint, TransformFeedbackLayoutDescriptorBuilder, TransformFeedbackAttributeDescriptor, TransformFeedbackAttributeType};
///
/// let mut builder = TransformFeedbackLayoutDescriptorBuilder::new(Some(TransformFeedbackLayoutAllocationHint {
///     bind_slot_count: 2,
///     attribute_count: 3
/// }));
///
/// builder.add_bind_slot()
///     .add_attribute(TransformFeedbackAttributeDescriptor {
///         name: "position".to_string(),
///         attribute_type: TransformFeedbackAttributeType::FloatVector3,
///         size: 1
///     })
///     .add_attribute(TransformFeedbackAttributeDescriptor {
///         name: "normal".to_string(),
///         attribute_type: TransformFeedbackAttributeType::FloatVector3,
///         size: 1
///     });
///
/// let layout_descriptor = builder.finish();
/// ```
pub struct TransformFeedbackLayoutDescriptorBuilder {
    layout: Vec<LayoutElement>,
}

impl TransformFeedbackLayoutDescriptorBuilder {
    /// Creates a new builder.
    ///
    /// Takes an optional `allocation_hint` to help reduce the number of allocations without over-
    /// allocating. With an accurate allocation hint the builder will only allocate once. See
    /// [TransformFeedbackLayoutAllocationHint] for details.
    pub fn new(allocation_hint: Option<TransformFeedbackLayoutAllocationHint>) -> Self {
        let layout = if let Some(hint) = allocation_hint {
            Vec::with_capacity((hint.bind_slot_count - 1 + hint.attribute_count) as usize)
        } else {
            Vec::new()
        };

        TransformFeedbackLayoutDescriptorBuilder { layout }
    }

    /// Adds a transform feedback buffer binding slot to the layout.
    pub fn add_buffer_slot(&mut self) -> TransformFeedbackBufferSlotAttributeAttacher {
        if self.layout.len() > 0 {
            self.layout.push(LayoutElement::NextBindSlot)
        }

        TransformFeedbackBufferSlotAttributeAttacher {
            layout_builder: self,
        }
    }

    /// Finishes building and returns the resulting [TransformFeedbackLayoutDescriptor].
    pub fn finish(self) -> TransformFeedbackLayoutDescriptor {
        TransformFeedbackLayoutDescriptor {
            layout: self.layout,
        }
    }
}

/// Returned from [TransformFeedbackLayoutDescriptorBuilder::add_buffer_slot], attaches
/// [TransformFeedbackAttributeDescriptor]s to transform feedback buffer bind slots.
pub struct TransformFeedbackBufferSlotAttributeAttacher<'a> {
    layout_builder: &'a mut TransformFeedbackLayoutDescriptorBuilder,
}

impl<'a> TransformFeedbackBufferSlotAttributeAttacher<'a> {
    /// Adds an attribute descriptor to the current bind slot.
    pub fn add_attribute(
        &mut self,
        attribute_descriptor: TransformFeedbackAttributeDescriptor,
    ) -> &mut TransformFeedbackBufferSlotAttributeAttacher<'a> {
        self.layout_builder
            .layout
            .push(LayoutElement::NextAttribute(attribute_descriptor));

        self
    }
}

/// Trait implemented by types that can be used as fields in structs that derive
/// [TransformFeedback].
///
/// # Unsafe
///
/// May only be implemented for a type if it is bitwise compatible with an array of GLSL type
/// [TYPE] with a length of [SIZE].
pub unsafe trait TransformFeedbackAttribute {
    /// The transform feedback attribute type associated with this type.
    const TYPE: TransformFeedbackAttributeType;

    /// The array size (length) associated with this type.
    ///
    /// Should always be `1` for non-array types.
    const SIZE: usize;
}

unsafe impl TransformFeedbackAttribute for f32 {
    const TYPE: TransformFeedbackAttributeType = TransformFeedbackAttributeType::Float;

    const SIZE: usize = 1;
}

unsafe impl TransformFeedbackAttribute for [f32; 2] {
    const TYPE: TransformFeedbackAttributeType = TransformFeedbackAttributeType::FloatVector2;

    const SIZE: usize = 1;
}

unsafe impl TransformFeedbackAttribute for [f32; 3] {
    const TYPE: TransformFeedbackAttributeType = TransformFeedbackAttributeType::FloatVector3;

    const SIZE: usize = 1;
}

unsafe impl TransformFeedbackAttribute for [f32; 4] {
    const TYPE: TransformFeedbackAttributeType = TransformFeedbackAttributeType::FloatVector4;

    const SIZE: usize = 1;
}

unsafe impl TransformFeedbackAttribute for [[f32; 2]; 2] {
    const TYPE: TransformFeedbackAttributeType = TransformFeedbackAttributeType::FloatMatrix2x2;

    const SIZE: usize = 1;
}

unsafe impl TransformFeedbackAttribute for [[f32; 3]; 2] {
    const TYPE: TransformFeedbackAttributeType = TransformFeedbackAttributeType::FloatMatrix2x3;

    const SIZE: usize = 1;
}

unsafe impl TransformFeedbackAttribute for [[f32; 4]; 2] {
    const TYPE: TransformFeedbackAttributeType = TransformFeedbackAttributeType::FloatMatrix2x4;

    const SIZE: usize = 1;
}

unsafe impl TransformFeedbackAttribute for [[f32; 2]; 3] {
    const TYPE: TransformFeedbackAttributeType = TransformFeedbackAttributeType::FloatMatrix3x2;

    const SIZE: usize = 1;
}

unsafe impl TransformFeedbackAttribute for [[f32; 3]; 3] {
    const TYPE: TransformFeedbackAttributeType = TransformFeedbackAttributeType::FloatMatrix3x3;

    const SIZE: usize = 1;
}

unsafe impl TransformFeedbackAttribute for [[f32; 4]; 3] {
    const TYPE: TransformFeedbackAttributeType = TransformFeedbackAttributeType::FloatMatrix3x4;

    const SIZE: usize = 1;
}

unsafe impl TransformFeedbackAttribute for [[f32; 2]; 4] {
    const TYPE: TransformFeedbackAttributeType = TransformFeedbackAttributeType::FloatMatrix4x2;

    const SIZE: usize = 1;
}

unsafe impl TransformFeedbackAttribute for [[f32; 3]; 4] {
    const TYPE: TransformFeedbackAttributeType = TransformFeedbackAttributeType::FloatMatrix4x3;

    const SIZE: usize = 1;
}

unsafe impl TransformFeedbackAttribute for [[f32; 4]; 4] {
    const TYPE: TransformFeedbackAttributeType = TransformFeedbackAttributeType::FloatMatrix4x4;

    const SIZE: usize = 1;
}

unsafe impl TransformFeedbackAttribute for i32 {
    const TYPE: TransformFeedbackAttributeType = TransformFeedbackAttributeType::Integer;

    const SIZE: usize = 1;
}

unsafe impl TransformFeedbackAttribute for [i32; 2] {
    const TYPE: TransformFeedbackAttributeType = TransformFeedbackAttributeType::IntegerVector2;

    const SIZE: usize = 1;
}

unsafe impl TransformFeedbackAttribute for [i32; 3] {
    const TYPE: TransformFeedbackAttributeType = TransformFeedbackAttributeType::IntegerVector3;

    const SIZE: usize = 1;
}

unsafe impl TransformFeedbackAttribute for [i32; 4] {
    const TYPE: TransformFeedbackAttributeType = TransformFeedbackAttributeType::IntegerVector4;

    const SIZE: usize = 1;
}

unsafe impl TransformFeedbackAttribute for u32 {
    const TYPE: TransformFeedbackAttributeType = TransformFeedbackAttributeType::UnsignedInteger;

    const SIZE: usize = 1;
}

unsafe impl TransformFeedbackAttribute for [u32; 2] {
    const TYPE: TransformFeedbackAttributeType =
        TransformFeedbackAttributeType::UnsignedIntegerVector2;

    const SIZE: usize = 1;
}

unsafe impl TransformFeedbackAttribute for [u32; 3] {
    const TYPE: TransformFeedbackAttributeType =
        TransformFeedbackAttributeType::UnsignedIntegerVector3;

    const SIZE: usize = 1;
}

unsafe impl TransformFeedbackAttribute for [u32; 4] {
    const TYPE: TransformFeedbackAttributeType =
        TransformFeedbackAttributeType::UnsignedIntegerVector4;

    const SIZE: usize = 1;
}

pub(crate) struct TransformFeedbackVaryings<'a>(pub &'a TransformFeedbackLayoutDescriptor);

impl<'a> Serialize for TransformFeedbackVaryings<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let TransformFeedbackVaryings(layout) = self;
        let layout = &layout.layout;

        let mut seq = serializer.serialize_seq(Some(layout.len()))?;

        for element in layout {
            match element {
                LayoutElement::NextAttribute(descriptor) => {
                    seq.serialize_element(&descriptor.name)?;
                }
                LayoutElement::NextBindSlot => {
                    seq.serialize_element("gl_NextBuffer")?;
                }
            }
        }

        seq.end()
    }
}
