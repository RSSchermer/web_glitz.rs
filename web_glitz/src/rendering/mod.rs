pub(crate) mod attachment;
pub use self::attachment::{
    AsAttachment, AsMultisampleAttachment, Attachment, MultisampleAttachment,
};

pub(crate) mod default_multisample_render_target;
pub use self::default_multisample_render_target::DefaultMultisampleRenderTarget;

pub(crate) mod default_render_target;
pub use self::default_render_target::DefaultRenderTarget;

pub(crate) mod framebuffer;
pub use self::framebuffer::{
    ActiveGraphicsPipeline, BindIndexBufferCommand, BindResourcesCommand, BindVertexBuffersCommand,
    BlitColorCompatible, BlitColorTarget, BlitCommand, BlitSource, BlitSourceDescriptor,
    BlitTargetDescriptor, DefaultDepthBuffer, DefaultDepthStencilBuffer, DefaultRGBABuffer,
    DefaultRGBBuffer, DefaultStencilBuffer, DepthBuffer, DepthStencilBuffer, DrawCommand,
    DrawIndexedCommand, FloatBuffer, Framebuffer, GraphicsPipelineTarget,
    GraphicsPipelineTaskBuilder, IntegerBuffer, ResolveColorCompatible,
    ResolveSource, ResolveSourceDescriptor, MultisampleFramebuffer,
    RenderingOutputBuffer, StencilBuffer, UnsignedIntegerBuffer,
};

mod render_pass;
pub use self::render_pass::{RenderPass, RenderPassContext};

pub(crate) mod encode_color_buffer;
pub use self::encode_color_buffer::{
    ColorBufferEncoding, ColorBufferEncodingContext, EncodeColorBuffer,
    EncodeMultisampleColorBuffer, FloatAttachment, IntegerAttachment, UnsignedIntegerAttachment,
};

pub(crate) mod encode_depth_stencil_buffer;
pub use self::encode_depth_stencil_buffer::{
    DepthAttachment, DepthStencilAttachment, DepthStencilBufferEncoding,
    DepthStencilBufferEncodingContext, EncodeDepthStencilBuffer,
    EncodeMultisampleDepthStencilBuffer, StencilAttachment,
};

pub(crate) mod render_target;
pub use self::render_target::{
    AttachColorFloat, AttachColorInteger, AttachColorUnsignedInteger, AttachDepth,
    AttachDepthStencil, AttachMultisampleColorFloat, AttachMultisampleDepth,
    AttachMultisampleDepthStencil, AttachStencil, MultisampleRenderTarget,
    MultisampleRenderTargetDescriptor, RenderTarget, RenderTargetDescriptor,
};

pub(crate) mod load_op;
pub use self::load_op::LoadOp;

mod store_op;
pub use self::store_op::StoreOp;
