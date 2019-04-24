pub(crate) mod attachable_image_ref;
pub use self::attachable_image_ref::AsAttachableImageRef;

pub(crate) mod default_render_target;
pub use self::default_render_target::DefaultRenderTarget;

pub(crate) mod render_target;
pub use self::render_target::RenderTarget;

pub(crate) mod render_target_attachment;
pub use self::render_target_attachment::{ColorAttachmentDescription, DepthStencilAttachmentDescription, AttachableImageRef, FloatAttachment, IntegerAttachment, UnsignedIntegerAttachment, DepthStencilAttachment, DepthAttachment, StencilAttachment, LoadOp, StoreOp};

pub(crate) mod render_target_description;
pub use self::render_target_description::RenderTargetDescription;

pub(crate) mod render_target_encoding;
pub use self::render_target_encoding::{EncodingContext, MaxColorAttachmentsExceeded, RenderTargetEncoder, RenderTargetEncoding};