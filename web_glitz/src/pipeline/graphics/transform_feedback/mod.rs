pub(crate) mod layout_descriptor;
pub use self::layout_descriptor::{
    TransformFeedback, TransformFeedbackAttribute, TransformFeedbackAttributeDescriptor,
    TransformFeedbackAttributeType, TransformFeedbackBufferSlotAttributeAttacher,
    TransformFeedbackBufferSlotAttributes, TransformFeedbackBufferSlotRef,
    TransformFeedbackBufferSlots, TransformFeedbackLayoutAllocationHint,
    TransformFeedbackLayoutDescriptor, TransformFeedbackLayoutDescriptorBuilder,
    TypedTransformFeedbackLayout, TransformFeedbackAttributeIdentifier
};

pub(crate) mod transform_feedback_buffers;
pub use self::transform_feedback_buffers::{
    TransformFeedbackBuffer, TransformFeedbackBuffers, TransformFeedbackBuffersEncoding,
    TransformFeedbackBuffersEncodingContext, TypedTransformFeedbackBuffer,
    TypedTransformFeedbackBuffers,
};
