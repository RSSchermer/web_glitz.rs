use crate::framebuffer::AsFramebufferAttachment;
use crate::image_format::{ColorRenderable, DepthRenderable, StencilRenderable};
use crate::renderbuffer::RenderbufferHandle;
use crate::texture::Texture2DLevel;

pub trait FramebufferDescriptor {
    type ColorAttachment0: ColorAttachable + AsFramebufferAttachment;

    type ColorAttachment1: ColorAttachable + AsFramebufferAttachment;

    type ColorAttachment2: ColorAttachable + AsFramebufferAttachment;

    type ColorAttachment3: ColorAttachable + AsFramebufferAttachment;

    type ColorAttachment4: ColorAttachable + AsFramebufferAttachment;

    type ColorAttachment5: ColorAttachable + AsFramebufferAttachment;

    type ColorAttachment6: ColorAttachable + AsFramebufferAttachment;

    type ColorAttachment7: ColorAttachable + AsFramebufferAttachment;

    type ColorAttachment8: ColorAttachable + AsFramebufferAttachment;

    type ColorAttachment9: ColorAttachable + AsFramebufferAttachment;

    type ColorAttachment10: ColorAttachable + AsFramebufferAttachment;

    type ColorAttachment11: ColorAttachable + AsFramebufferAttachment;

    type ColorAttachment12: ColorAttachable + AsFramebufferAttachment;

    type ColorAttachment13: ColorAttachable + AsFramebufferAttachment;

    type ColorAttachment14: ColorAttachable + AsFramebufferAttachment;

    type ColorAttachment15: ColorAttachable + AsFramebufferAttachment;

    type DepthAttachment: DepthAttachable + AsFramebufferAttachment;

    type StencilAttachment: StencilAttachable + AsFramebufferAttachment;

    fn color_attachment_0(&self) -> &Self::ColorAttachment0;

    fn color_attachment_1(&self) -> &Self::ColorAttachment1;

    fn color_attachment_2(&self) -> &Self::ColorAttachment2;

    fn color_attachment_3(&self) -> &Self::ColorAttachment3;

    fn color_attachment_4(&self) -> &Self::ColorAttachment4;

    fn color_attachment_5(&self) -> &Self::ColorAttachment5;

    fn color_attachment_6(&self) -> &Self::ColorAttachment6;

    fn color_attachment_7(&self) -> &Self::ColorAttachment7;

    fn color_attachment_8(&self) -> &Self::ColorAttachment8;

    fn color_attachment_9(&self) -> &Self::ColorAttachment9;

    fn color_attachment_10(&self) -> &Self::ColorAttachment10;

    fn color_attachment_11(&self) -> &Self::ColorAttachment11;

    fn color_attachment_12(&self) -> &Self::ColorAttachment12;

    fn color_attachment_13(&self) -> &Self::ColorAttachment13;

    fn color_attachment_14(&self) -> &Self::ColorAttachment14;

    fn color_attachment_15(&self) -> &Self::ColorAttachment15;

    fn depth_attachment(&self) -> &Self::DepthAttachment;

    fn stencil_attachment(&self) -> &Self::StencilAttachment;
}

pub unsafe trait ColorAttachable {}

unsafe impl<F> ColorAttachable for Texture2DLevel<F> where F: ColorRenderable {}

unsafe impl<F> ColorAttachable for RenderbufferHandle<F> where F: ColorRenderable {}

pub unsafe trait DepthAttachable {}

unsafe impl<F> DepthAttachable for Texture2DLevel<F> where F: DepthRenderable {}

unsafe impl<F> DepthAttachable for RenderbufferHandle<F> where F: DepthRenderable {}

pub unsafe trait StencilAttachable {}

unsafe impl<F> StencilAttachable for Texture2DLevel<F> where F: StencilRenderable {}

unsafe impl<F> StencilAttachable for RenderbufferHandle<F> where F: StencilRenderable {}
