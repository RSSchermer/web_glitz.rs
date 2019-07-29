mod blending;
pub use self::blending::{BlendEquation, BlendFactor, Blending};

mod descriptor;
pub use self::descriptor::{
    GraphicsPipelineDescriptor, GraphicsPipelineDescriptorBuilder, SlotBindingStrategy,
};

mod fragment_test;
pub use self::fragment_test::{
    DepthRange, DepthTest, PolygonOffset, StencilOperation, StencilTest, TestFunction,
};

pub(crate) mod graphics_pipeline;
pub use self::graphics_pipeline::{GraphicsPipeline, ShaderLinkingError};

pub(crate) mod primitive_assembly;
pub use self::primitive_assembly::{CullingMode, LineWidth, PrimitiveAssembly, WindingOrder};

pub(crate) mod shader;
pub use self::shader::{FragmentShader, VertexShader};

pub(crate) mod transform_feedback;
pub use self::transform_feedback::{
    TransformFeedback, TransformFeedbackAttribute, TransformFeedbackAttributeDescriptor,
    TransformFeedbackAttributeType, TransformFeedbackBuffer,
    TransformFeedbackBufferSlotAttributeAttacher, TransformFeedbackBufferSlotAttributes,
    TransformFeedbackBufferSlotRef, TransformFeedbackBufferSlots, TransformFeedbackBuffers,
    TransformFeedbackBuffersEncoding, TransformFeedbackBuffersEncodingContext,
    TransformFeedbackLayoutAllocationHint, TransformFeedbackLayoutDescriptor,
    TransformFeedbackLayoutDescriptorBuilder, TypedTransformFeedbackBuffer,
    TypedTransformFeedbackBuffers, TypedTransformFeedbackLayout,
};

pub(crate) mod vertex;
pub use self::vertex::{
    attribute_format, IncompatibleVertexInputLayout, IndexBuffer, IndexBufferEncoding,
    IndexBufferEncodingContext, IndexFormat, IndexType, InputRate, TypedVertexBuffer,
    TypedVertexBuffers, TypedVertexInputLayout, Vertex, VertexAttributeDescriptor,
    VertexAttributeType, VertexBuffer, VertexBufferSlotAttributeAttacher, VertexBufferSlotRef,
    VertexBuffers, VertexBuffersEncoding, VertexBuffersEncodingContext,
    VertexInputLayoutAllocationHint, VertexInputLayoutDescriptor,
    VertexInputLayoutDescriptorBuilder,
};

mod viewport;
pub use self::viewport::Viewport;

pub(crate) mod util;

pub struct Untyped(());
