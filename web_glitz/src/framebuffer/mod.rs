mod framebuffer_descriptor;
pub use self::framebuffer_descriptor::{
    ColorAttachable, DepthAttachable, FramebufferDescriptor, StencilAttachable,
};

mod framebuffer_descriptor_builder;
pub use self::framebuffer_descriptor_builder::{
    BuildFramebufferDescriptor, FramebufferDescriptorBuilder,
};

pub(crate) mod framebuffer_handle;
pub use self::framebuffer_handle::{
    AsFramebufferAttachment, FramebufferAttachment, FramebufferHandle,
};

mod render_pass;
pub use self::render_pass::{DrawBuffer, RenderPass, RenderPassContext, SubPass, SubPassContext};
