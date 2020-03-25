//! A render pass (see [render_pass]) needs a render target: zero or more color images and zero or
//! one depth-stencil image (see also [image]) into which the output of the render pass may be
//! stored when the render pass completes. These images are said to be "attached" to the render
//! target and are referred to as the "attached images" or simply the "attachments".

pub(crate) mod attachable_image_ref;
pub use self::attachable_image_ref::{AsAttachableImageRef, AttachableImageRef};

pub(crate) mod default_render_target;
pub use self::default_render_target::DefaultRenderTarget;

pub(crate) mod render_target_attachment;
pub use self::render_target_attachment::{
    ColorAttachmentDescription, ColorAttachmentEncoding, ColorAttachmentEncodingContext,
    DepthAttachment, DepthStencilAttachment, DepthStencilAttachmentDescription,
    DepthStencilAttachmentEncoding, DepthStencilAttachmentEncodingContext, FloatAttachment,
    IntegerAttachment, LoadOp, StencilAttachment, StoreOp, UnsignedIntegerAttachment,
};

pub(crate) mod render_target_description;
pub use self::render_target_description::{RenderTargetDescriptor, RenderTargetDescription};
