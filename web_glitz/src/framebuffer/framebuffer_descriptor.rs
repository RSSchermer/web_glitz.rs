use crate::framebuffer::AsFramebufferAttachment;
use crate::image_format::{ColorRenderable, DepthRenderable, StencilRenderable};
use crate::renderbuffer::{RenderbufferData, RenderbufferHandle};
use crate::rendering_context::{RenderingContext};
use crate::texture::{Texture2DImageRef, TextureImageData};

pub trait FramebufferDescriptor<Rc>
    where
        Rc: RenderingContext,
{
    type ColorAttachment0: ColorAttachable + AsFramebufferAttachment<Rc>;

    type ColorAttachment1: ColorAttachable + AsFramebufferAttachment<Rc>;

    type ColorAttachment2: ColorAttachable + AsFramebufferAttachment<Rc>;

    type ColorAttachment3: ColorAttachable + AsFramebufferAttachment<Rc>;

    type ColorAttachment4: ColorAttachable + AsFramebufferAttachment<Rc>;

    type ColorAttachment5: ColorAttachable + AsFramebufferAttachment<Rc>;

    type ColorAttachment6: ColorAttachable + AsFramebufferAttachment<Rc>;

    type ColorAttachment7: ColorAttachable + AsFramebufferAttachment<Rc>;

    type ColorAttachment8: ColorAttachable + AsFramebufferAttachment<Rc>;

    type ColorAttachment9: ColorAttachable + AsFramebufferAttachment<Rc>;

    type ColorAttachment10: ColorAttachable + AsFramebufferAttachment<Rc>;

    type ColorAttachment11: ColorAttachable + AsFramebufferAttachment<Rc>;

    type ColorAttachment12: ColorAttachable + AsFramebufferAttachment<Rc>;

    type ColorAttachment13: ColorAttachable + AsFramebufferAttachment<Rc>;

    type ColorAttachment14: ColorAttachable + AsFramebufferAttachment<Rc>;

    type ColorAttachment15: ColorAttachable + AsFramebufferAttachment<Rc>;

    type DepthAttachment: DepthAttachable + AsFramebufferAttachment<Rc>;

    type StencilAttachment: StencilAttachable + AsFramebufferAttachment<Rc>;

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

unsafe impl<F, Rc> ColorAttachable for Texture2DImageRef<F, Rc>
    where
        F: ColorRenderable,
        Rc: RenderingContext,
{
}

unsafe impl<F, Rc> ColorAttachable for RenderbufferHandle<F, Rc>
    where
        F: ColorRenderable,
        Rc: RenderingContext,
{
}

pub unsafe trait DepthAttachable {}

unsafe impl<F, Rc> DepthAttachable for Texture2DImageRef<F, Rc>
    where
        F: DepthRenderable,
        Rc: RenderingContext,
{
}

unsafe impl<F, Rc> DepthAttachable for RenderbufferHandle<F, Rc>
    where
        F: DepthRenderable,
        Rc: RenderingContext,
{
}

pub unsafe trait StencilAttachable {}

unsafe impl<F, Rc> StencilAttachable for Texture2DImageRef<F, Rc>
    where
        F: StencilRenderable,
        Rc: RenderingContext,
{
}

unsafe impl<F, Rc> StencilAttachable for RenderbufferHandle<F, Rc>
    where
        F: StencilRenderable,
        Rc: RenderingContext,
{
}
